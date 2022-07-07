mod env;
mod err;
mod webhook;

use hyper::{
	service::{make_service_fn, service_fn},
	Body, Error, Request, Response, Server, StatusCode,
};
use lazy_static::lazy_static;
use std::{
	collections::{
		hash_map::Entry::{Occupied, Vacant},
		HashMap,
	},
	net::SocketAddr,
	time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
	sync::Mutex,
	task::{self, JoinHandle},
	time::sleep,
};

use crate::{
	env::{get_env, get_env_default, DEFAULT_DELIVER_DURATION, DEFAULT_PORT},
	err::form_error_res,
	webhook::{parse_body, validate_request, WebhookBatch, WebhookParts},
};

lazy_static! {
	static ref TASK_MAP: Mutex<HashMap<WebhookParts, JoinHandle<()>>> = Mutex::new(HashMap::new());
	static ref BATCH_MAP: Mutex<HashMap<WebhookParts, WebhookBatch>> = Mutex::new(HashMap::new());
	static ref DELIVER_DURATION: Duration = {
		let duration: Option<u64> = get_env("DELIVER_MS", false);
		if duration.is_some() {
			return Duration::from_millis(duration.unwrap());
		}

		return DEFAULT_DELIVER_DURATION;
	};
}

async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Error> {
	let (parts, body) = request.into_parts();
	let webhook_parts = match validate_request(&parts) {
		Ok(parts) => parts,
		Err(err) => {
			// completely arbitrary error codes, but should stick
			let response = form_error_res(err.code, 100, &err.message);
			return Ok(response);
		}
	};

	let payload = match parse_body(body).await {
		Ok(payload) => payload,
		Err(err) => {
			let response = form_error_res(StatusCode::BAD_REQUEST, 101, err);
			return Ok(response);
		}
	};

	let mut batch_map = BATCH_MAP.lock().await;
	let batch = match batch_map.entry(webhook_parts.clone()) {
		Occupied(entry) => {
			let batch = entry.into_mut();
			batch.payloads.push(payload);
			batch.clone()
		}
		Vacant(entry) => {
			let key = entry.key().clone();
			let batch = entry.insert(WebhookBatch {
				created: SystemTime::now(),
				payloads: vec![payload],
				parts: key,
			});

			batch.clone()
		}
	};

	let mut task_map = TASK_MAP.lock().await;
	let task = task_map.entry(webhook_parts.clone());
	if let Occupied(mut entry) = task {
		let task = entry.get_mut();
		task.abort();
		entry.remove();
		task_map.remove(&webhook_parts);
	};

	let task_batch = batch.clone();
	let task_parts = webhook_parts.clone();
	let task = task::spawn(async move {
		sleep(*DELIVER_DURATION).await;
		let mut task_map = TASK_MAP.lock().await;
		let mut batch_map = BATCH_MAP.lock().await;

		webhook::deliver(task_batch).await;
		batch_map.remove(&task_parts);
		task_map.remove(&task_parts);
	});

	task_map.insert(webhook_parts, task);
	let since_epoch = batch.created.duration_since(UNIX_EPOCH).unwrap();
	let response = Response::builder()
		.status(StatusCode::NO_CONTENT)
		.header(
			"X-Batch-Id",
			format!("{}-{}", batch.parts.webhook_id, batch.parts.webhook_token),
		)
		.header("X-Batch-Size", batch.payloads.len())
		.header("X-Batch-Created", since_epoch.as_millis().to_string())
		.body(Body::empty())
		.unwrap();
	return Ok(response);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let port: u16 = get_env_default("PORT", DEFAULT_PORT);
	let addr = SocketAddr::from(([127, 0, 0, 1], port));
	let make_svc = make_service_fn(|_conn| async { Ok::<_, Error>(service_fn(handle_request)) });
	let server = Server::bind(&addr).serve(make_svc);
	println!("Server listening at {}", server.local_addr());
	server.await?;
	Ok(())
}
