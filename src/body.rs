use hyper::Body;

use crate::webhook::WebhookPayload;

/// Parses a request body and returns the WebhookPayload if valid.
pub async fn parse_body<'a>(body: Body) -> Result<WebhookPayload, &'a str> {
	let string = body_to_string(body).await?;
	let payload: WebhookPayload = match serde_json::from_str(string.as_str()) {
		Ok(payload) => payload,
		Err(_) => return Err("Invalid JSON."),
	};

	return Ok(payload);
}

/// Parses request body into a string.
pub async fn body_to_string<'a>(body: Body) -> Result<String, &'a str> {
	let string = match hyper::body::to_bytes(body).await {
		Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
			Ok(string) => string,
			Err(_) => return Err("Invalid UTF-8."),
		},
		Err(_) => return Err("Failed to parse body."),
	};

	return Ok(string);
}

/// Merges a vector of WebhookPayloads into a single body.
pub fn merge_body(payloads: &Vec<WebhookPayload>) -> Result<Body, String> {
	let mut aggr = WebhookPayload {
		content: None,
		username: None,
		avatar_url: None,
		embeds: None,
		tts: None,
		allowed_mentions: None,
		components: None,
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

		if aggr.allowed_mentions.is_none() && payload.allowed_mentions.is_some() {
			aggr.allowed_mentions = Some(payload.allowed_mentions.unwrap());
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

		if payload.embeds.is_some() {
			if aggr.embeds.is_none() {
				aggr.embeds = Some(payload.embeds.unwrap());
			} else {
				let mut embeds = aggr.embeds.unwrap();
				embeds.extend(payload.embeds.unwrap());
				aggr.embeds = Some(embeds);
			}
		}

		if payload.components.is_some() {
			if aggr.components.is_none() {
				aggr.components = Some(payload.components.unwrap());
			} else {
				let mut components = aggr.components.unwrap();
				components.extend(payload.components.unwrap());
				aggr.components = Some(components);
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
