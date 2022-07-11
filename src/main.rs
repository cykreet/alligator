mod env;
mod err;
mod shutdown;
mod webhook;

use crate::{
	env::{get_env, get_env_default, DEFAULT_DELIVER_DURATION, DEFAULT_EMBED_LIMIT, DEFAULT_PORT},
	err::form_error_res,
	shutdown::Shutdown,
	webhook::{parse_body, validate_request, WebhookBatch, WebhookParts, WebhookPayload},
};

use async_recursion::async_recursion;
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
	sync::{
		broadcast,
		mpsc::{self, Sender},
		Mutex, MutexGuard,
	},
	task::{self, JoinHandle},
	time::sleep,
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
	static ref EMBED_LIMIT: u8 = {
		let limit: Option<u8> = get_env("EMBED_LIMIT", false);
		if limit.is_some() {
			return limit.unwrap();
		}

		return DEFAULT_EMBED_LIMIT;
	};
}

#[async_recursion]
async fn handle_request(
	payload: WebhookPayload,
	webhook_parts: WebhookParts,
	batch_map: &mut MutexGuard<HashMap<WebhookParts, WebhookBatch>>,
	task_map: &mut MutexGuard<HashMap<WebhookParts, JoinHandle<()>>>,
	mut shutdown: Shutdown,
	_shutdown_complete: Sender<()>,
) -> Result<Response<Body>, Error> {
	let task = task_map.entry(webhook_parts.clone());
	if let Occupied(mut entry) = task {
		let task = entry.get_mut();
		task.abort();
		entry.remove();
		task_map.remove(&webhook_parts);
	};

	let batch_payload = payload.clone();
	let batch = match batch_map.entry(webhook_parts.clone()) {
		Occupied(entry) => {
			let batch = entry.into_mut();
			let embeds = batch
				.payloads
				.iter()
				.filter(|payload| payload.embeds.is_some())
				.flat_map(|payload| payload.embeds.as_ref().unwrap())
				.collect::<Vec<_>>();
			if embeds.len() >= *EMBED_LIMIT as usize {
				let shutdown_complete = _shutdown_complete.clone();
				webhook::deliver(batch.clone(), shutdown_complete).await;
				batch_map.remove(&webhook_parts);
				task_map.remove(&webhook_parts);
				return handle_request(
					payload,
					webhook_parts,
					batch_map,
					task_map,
					shutdown,
					_shutdown_complete,
				)
				.await;
			}

			batch.payloads.push(batch_payload);
			batch.clone()
		}
		Vacant(entry) => {
			let key = entry.key().clone();
			let batch = entry.insert(WebhookBatch {
				created: SystemTime::now(),
				payloads: vec![batch_payload],
				parts: key,
			});

			batch.clone()
		}
	};

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

	let task_parts = webhook_parts.clone();
	let task = task::spawn(async move {
		if shutdown.is_shutdown() {
			return;
		}

		tokio::select! {
			_ = sleep(*DELIVER_DURATION) => {
				let mut task_map = TASK_MAP.lock().await;
				let mut batch_map = BATCH_MAP.lock().await;
				webhook::deliver(batch, _shutdown_complete).await;
				batch_map.remove(&task_parts);
				task_map.remove(&task_parts);

			},
			_ = shutdown.recv() => {
				let mut task_map = TASK_MAP.lock().await;
				let mut batch_map = BATCH_MAP.lock().await;
				webhook::deliver(batch, _shutdown_complete).await;
				batch_map.remove(&task_parts);
				task_map.remove(&task_parts);
			}
		};
	});

	task_map.insert(webhook_parts, task);
	return Ok(response);
}

async fn forward_request(
	request: Request<Body>,
	shutdown: Shutdown,
	_shutdown_complete: Sender<()>,
) -> Result<Response<Body>, Error> {
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

	if payload.embeds.is_some() && payload.embeds.as_ref().unwrap().len() > *EMBED_LIMIT as usize {
		// note: could be neat to queue up payloads and send them later - project for another day,
		// could be relevant: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
		let response = form_error_res(StatusCode::BAD_REQUEST, 102, "Too many embeds");
		return Ok(response);
	}

	let mut task_map = TASK_MAP.lock().await;
	let mut batch_map = BATCH_MAP.lock().await;
	return handle_request(
		payload,
		webhook_parts,
		&mut batch_map,
		&mut task_map,
		shutdown,
		_shutdown_complete,
	)
	.await;
}

#[tokio::main]
async fn main() {
	let (notify_shutdown, _) = broadcast::channel(1);
	let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);
	let port: u16 = get_env_default("LISTEN_PORT", DEFAULT_PORT);
	let addr = SocketAddr::from(([0, 0, 0, 0], port));
	let make_service = make_service_fn(|_conn| {
		let shutdown_compelete_tx = shutdown_complete_tx.clone();
		let notify_shutdown = notify_shutdown.clone();
		let service = service_fn(move |request| {
			forward_request(
				request,
				Shutdown::new(notify_shutdown.subscribe()),
				shutdown_compelete_tx.clone(),
			)
		});
		async move { Ok::<_, Error>(service) }
	});

	let server = Server::bind(&addr).serve(make_service);
	println!("Server listening at http://{}", server.local_addr());
	let graceful = server.with_graceful_shutdown(async {
		tokio::signal::ctrl_c()
			.await
			.expect("Failed to install SIGINT handler.");
	});

	tokio::select! {
		_ = graceful => {
			println!("Received SIGINT, exiting.");
		},
	}

	drop(notify_shutdown);
	drop(shutdown_complete_tx);
	// instead of sending messages to all tasks that need finishing, we wait
	// for the channel to be closed, which happens when every sender has been dropped
	// https://tokio.rs/tokio/topics/shutdown
	let _ = shutdown_complete_rx.recv().await;
}
