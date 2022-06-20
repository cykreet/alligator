use hyper::{Body, Client, Method, Request};
use lazy_static::lazy_static;
use std::{
	collections::{
		hash_map::Entry::{Occupied, Vacant},
		HashMap,
	},
	time::SystemTime,
};
use tokio::sync::Mutex;

use crate::{
	env::{get_env_default, DISCORD_WEBHOOK_ENDPOINT},
	webhook::{WebhookBatch, WebhookParts, WebhookPayload},
};

lazy_static! {
	static ref WEBHOOK_BATCHES: Mutex<HashMap<u64, WebhookBatch>> = Mutex::new(HashMap::new());
}

pub fn form_response(code: u8, message: &str) -> Body {
	return format!("{{ code: {}, message: {} }}", code, message).into();
}

pub async fn get_insert(hash: u64, parts: WebhookParts, payload: WebhookPayload) -> WebhookBatch {
	let mut batch_map = WEBHOOK_BATCHES.lock().await;
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
				parts,
			});
			batch.clone()
		}
	};

	return batch;
}

pub async fn deliver(hash: u64) -> () {
	let mut batch_map = WEBHOOK_BATCHES.lock().await;
	let batch = match batch_map.remove(&hash) {
		Some(batch) => batch,
		None => {
			return eprintln!("No batch found for hash: {}", hash);
		}
	};

	let client = Client::new();
	let uri: String = get_env_default(
		"DISCORD_WEBHOOK_ENDPOINT",
		DISCORD_WEBHOOK_ENDPOINT.to_string(),
	);
	let request = Request::builder()
		.uri(uri)
		.method(Method::POST)
		.header("Content-Type", "application/json")
		// todo: merge body
		.body(Body::empty())
		.unwrap();

	let response = client.request(request).await;
	if response.is_err() {
		eprintln!("Failed to deliver batch {}.", hash);
	}
}
