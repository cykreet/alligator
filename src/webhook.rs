use hyper::{header::HeaderValue, http::request::Parts, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, time::SystemTime};

use crate::env::{get_env_default, DISCORD_WEBHOOK_ENDPOINT};

#[derive(Clone)]
pub struct WebhookBatch {
	pub created: SystemTime,
	pub parts: WebhookParts,
	pub payloads: Vec<WebhookPayload>,
}

#[derive(Hash, Clone, Eq, PartialEq)]
pub struct WebhookParts {
	pub webhook_id: String,
	pub webhook_token: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
	pub content: Option<String>,
	pub username: Option<String>,
	pub avatar_url: Option<String>,
	pub embeds: Option<Vec<String>>,
	pub tts: Option<bool>,
	pub thread_name: Option<String>,
}

const PATH_RE: &str = r"/api/(v[0-9]{1,3}/)?webhooks/(?P<webhook_id>[0-9]\w+)/(?P<webhook_token>[A-z0-9-]{1,100})(\?(?P<params>[A-z0-9-\.=&]{1,50}))?";

lazy_static! {
	static ref PATH_REGEX: Regex = Regex::new(PATH_RE).unwrap();
}

/// Formats an error code and message into a JSON object.
pub fn form_error_body(code: u8, message: &str) -> Body {
	return format!("{{ code: {}, message: \"{}\" }}", code, message).into();
}

/// Validates a request and returns the WebhookParts if valid.
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
	let caps = match PATH_REGEX.captures(path) {
		Some(caps) => caps,
		None => return Err("Invalid path."),
	};

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

/// Parses a request body and returns the WebhookPayload if valid.
pub async fn parse_body<'a>(body: Body) -> Result<WebhookPayload, &'a str> {
	let bytes = match hyper::body::to_bytes(body).await {
		Ok(bytes) => bytes,
		Err(_) => return Err("Failed to parse body."),
	};

	let string = match std::str::from_utf8(&bytes) {
		Ok(string) => string,
		Err(_) => return Err("Invalid UTF-8."),
	};

	let payload: WebhookPayload = match serde_json::from_str(string).ok() {
		Some(payload) => payload,
		None => return Err("Invalid JSON."),
	};

	return Ok(payload);
}

/// Delivers a batch of payloads to a webhook.
pub async fn deliver(batch: WebhookBatch) -> () {
	let host: String = get_env_default(
		"DISCORD_WEBHOOK_ENDPOINT",
		DISCORD_WEBHOOK_ENDPOINT.to_string(),
	);

	let uri = format!(
		"{}{}/{}",
		host, batch.parts.webhook_id, batch.parts.webhook_token
	);

	let body = match merge_body(&batch.payloads) {
		Ok(body) => body,
		Err(err) => {
			return eprintln!("Failed to merge batch payload:\n{}", err);
		}
	};

	let request = Request::builder()
		.uri(uri)
		.method(Method::POST)
		.header("Content-Type", "application/json")
		.body(body)
		.unwrap();
	let https = HttpsConnector::new();
	let client = Client::builder().build(https);
	let response = client.request(request).await;
	if let Err(err) = response {
		eprintln!("Failed to deliver batch:\n{}", err);
	}
}

/// Merges a vector of WebhookPayloads into a single body.
pub fn merge_body(payloads: &Vec<WebhookPayload>) -> Result<Body, String> {
	let mut aggr = WebhookPayload {
		content: None,
		username: None,
		avatar_url: None,
		embeds: None,
		tts: None,
		thread_name: None,
	};

	for payload in payloads {
		let payload = payload.clone();
		if aggr.username.is_none() && payload.username.is_some() {
			aggr.username = Some(payload.username.unwrap());
		}

		if aggr.avatar_url.is_none() && payload.avatar_url.is_some() {
			aggr.avatar_url = Some(payload.avatar_url.unwrap());
		}

		if aggr.tts.is_none() && payload.tts.is_some() {
			aggr.tts = Some(payload.tts.unwrap());
		}

		if aggr.thread_name.is_none() && payload.thread_name.is_some() {
			aggr.thread_name = Some(payload.thread_name.unwrap());
		}

		if payload.content.is_some() {
			if aggr.content.is_none() {
				aggr.content = Some(payload.content.unwrap());
			} else {
				let content = format!(
					"{}\n{}",
					aggr.content.unwrap(),
					payload.content.as_ref().unwrap()
				);
				aggr.content = Some(content);
			}
		}
	}

	let body = match serde_json::to_string(&aggr) {
		Ok(body) => body,
		Err(err) => {
			return Err(err.to_string());
		}
	};

	return Ok(body.into());
}
