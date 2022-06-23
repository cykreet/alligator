mod env;
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
	time::{Duration, SystemTime},
};
use tokio::{
	sync::Mutex,
	task::{self, JoinHandle},
	time::sleep,
};

use crate::{
	env::{get_env, get_env_default, DEFAULT_DELIVER_DURATION, DEFAULT_PORT},
	webhook::{
		deliver, form_response, hash_parts, read_body, validate_request, WebhookBatch, WebhookPayload,
	},
};

lazy_static! {
	static ref TASK_MAP: Mutex<HashMap<u64, JoinHandle<()>>> = Mutex::new(HashMap::new());
	static ref BATCH_MAP: Mutex<HashMap<u64, WebhookBatch>> = Mutex::new(HashMap::new());
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
			let body = form_response(100, err);
			let response = Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.header("Content-Type", "application/json")
				.body(body)
				.unwrap();
			return Ok(response);
		}
	};

	let full_body = match read_body(body).await {
		Ok(body) => body,
		Err(err) => {
			let body = form_response(101, err);
			let response = Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(body)
				.unwrap();
			return Ok(response);
		}
	};

	let hash = hash_parts(&webhook_parts);
	let payload: WebhookPayload = serde_json::from_str(&full_body).unwrap();
	let mut batch_map = BATCH_MAP.lock().await;
	let batch = match batch_map.entry(hash) {
		Occupied(entry) => {
			let batch = entry.into_mut();
			batch.payloads.push(payload);
			batch.clone()
		}
		Vacant(entry) => {
			let batch = entry.insert(WebhookBatch {
				created: SystemTime::now(),
				payloads: vec![payload],
				parts: webhook_parts,
			});

			batch.clone()
		}
	};

	let mut task_map = TASK_MAP.lock().await;
	let task = task_map.entry(hash);
	if let Occupied(mut entry) = task {
		let task = entry.get_mut();
		task.abort();
		entry.remove();
	}

	let task_batch = batch.clone();
	let task = task::spawn(async move {
		sleep(*DELIVER_DURATION).await;
		let mut task_map = TASK_MAP.lock().await;
		let mut batch_map = BATCH_MAP.lock().await;

		deliver(task_batch).await;
		batch_map.remove(&hash);
		task_map.remove(&hash);
	});

	task_map.insert(hash, task);
	let response = Response::builder()
		.status(StatusCode::OK)
		.header("X-Batch-Id", hash)
		.header("X-Batch-Size", batch.payloads.len())
		.body(Body::empty())
		.unwrap();
	return Ok(response);
}

#[tokio::main]
async fn main() {
	let port: u16 = get_env_default("PORT", DEFAULT_PORT);
	let addr = SocketAddr::from(([127, 0, 0, 1], port));
	let make_svc = make_service_fn(|_conn| async { Ok::<_, Error>(service_fn(handle_request)) });
	let server = Server::bind(&addr).serve(make_svc);
	println!("Server listening at {}", server.local_addr());
	if let Err(err) = server.await {
		eprintln!("Something went wrong: {}", err);
	}
}
