mod env;
mod payload;
mod webhook;

use hyper::{
	service::{make_service_fn, service_fn},
	Body, Error, Request, Response, Server, StatusCode,
};
use lazy_static::lazy_static;
use std::{net::SocketAddr, time::Duration};
use tokio::task;

use crate::{
	env::{get_env, get_env_default, DEFAULT_EXPIRES_DURATION, DEFAULT_PORT},
	payload::form_response,
	webhook::{hash_parts, read_body, validate_request, WebhookPayload},
};

lazy_static! {
	static ref EXPIRES_DURATION: Duration = {
		let duration: Option<u64> = get_env("EXPIRES_DURATION", false);
		if duration.is_some() {
			return Duration::from_millis(duration.unwrap());
		}

		return DEFAULT_EXPIRES_DURATION;
	};
}

async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Error> {
	let (parts, body) = request.into_parts();
	let full_body = match read_body(body).await {
		Ok(body) => body,
		Err(err) => {
			// completely arbitrary error codes, but should stick
			let body = form_response(100, err);
			let response = Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(body)
				.unwrap();
			return Ok(response);
		}
	};

	let webhook_parts = match validate_request(&parts) {
		Ok(val) => val,
		Err(err) => {
			let body = form_response(102, err);
			let response = Response::builder()
				.status(StatusCode::BAD_REQUEST)
				.body(body)
				.unwrap();
			return Ok(response);
		}
	};

	let hash = hash_parts(&webhook_parts);
	let payload: WebhookPayload = serde_json::from_str(&full_body).unwrap();
	let batch = payload::get_insert(hash, webhook_parts, payload).await;
	let task = task::spawn(async move {
		payload::deliver(hash).await;
	});

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
