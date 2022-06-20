use futures::TryStreamExt;
use hyper::{header::HeaderValue, http::request::Parts, Body, Method};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
	time::SystemTime,
};

#[derive(Clone)]
pub struct WebhookBatch {
	pub created: SystemTime,
	pub parts: WebhookParts,
	pub payloads: Vec<WebhookPayload>,
}

#[derive(Hash, Clone)]
pub struct WebhookParts {
	pub webhook_id: String,
	pub webhook_token: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WebhookPayload {
	content: String,
	username: String,
	avatar_url: String,
	tts: bool,
}

const BODY_LIMIT: u16 = 32000; // 4KB
const PATH_RE: &str = r"/(http?s?://)?[A-z0-9.:\-_]{1,253}/api/(v[0-9]{1,3}/)?webhooks/(?P<webhook_id>[0-9]\w+)/(?P<webhook_token>[A-z0-9-]{1,100})(\?(?P<params>[A-z0-9-\.=&]{1,50}))?/";

lazy_static! {
	static ref PATH_REGEX: Regex = Regex::new(PATH_RE).unwrap();
}

pub async fn read_body<'a>(body: Body) -> Result<String, &'a str> {
	let mut seen: usize = 0;
	let full_body = body
		.try_fold(Vec::new(), |mut data, chunk| async move {
			seen += chunk.len();
			if seen > BODY_LIMIT as usize {
				// not great, but i couldn't find a way to initialise
				// a hyper error without redoing it entirely
				return Ok(Vec::new());
				// return Err("Body too large".into());
			}

			data.extend_from_slice(&chunk);
			return Ok(data);
		})
		.await;

	let body = full_body.unwrap();
	if body.is_empty() {
		return Err("Invalid body size.");
	}

	let string = String::from_utf8(body).unwrap();
	return Ok(string);
}

pub fn validate_request(parts: &Parts) -> Result<WebhookParts, &str> {
	if parts.method != Method::POST {
		return Err("Method not supported.");
	}

	let content_type = parts.headers.get("content-type");
	let header_value = HeaderValue::from_static("application/json");
	if content_type != Some(&header_value) {
		return Err("Expected \"Content-Type\" header to be one of {'application/json', 'application/x-www-form-urlencoded', 'multipart/form-data'}.");
	}

	let path = parts.uri.path();
	let caps = PATH_REGEX.captures(path).unwrap();
	let webhook_id = caps.name("webhook_id");
	if webhook_id.is_none() {
		return Err("Webhook ID could not be indentified.");
	}

	let webhook_token = caps.name("webhook_token");
	if webhook_token.is_none() {
		return Err("Webhook token could not be indentified.");
	}

	// todo: params
	return Ok(WebhookParts {
		webhook_id: webhook_id.unwrap().as_str().to_string(),
		webhook_token: webhook_token.unwrap().as_str().to_string(),
	});
}

pub fn hash_parts(parts: &WebhookParts) -> u64 {
	let mut hasher = DefaultHasher::new();
	parts.hash(&mut hasher);
	return hasher.finish();
}
