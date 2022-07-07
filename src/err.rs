use hyper::{Body, Response, StatusCode};

pub struct ValidateError {
	pub code: StatusCode,
	pub message: String,
}

impl ValidateError {
	pub fn new(code: StatusCode, message: String) -> Self {
		Self { code, message }
	}
}

/// Forms a JSON error response from a code and message.
pub fn form_error_res(http_code: StatusCode, code: u8, message: &str) -> Response<Body> {
	let body = format!("{{ code: {}, message: \"{}\" }}", code, message).into();
	return Response::builder()
		.status(http_code)
		.header("Content-Type", "application/json")
		.body(body)
		.unwrap();
}
