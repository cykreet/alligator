mod body;
mod env;
mod payload;
mod webhook;

use body::read_body;
use env::get_env;
use hyper::{
  service::{make_service_fn, service_fn},
  Body, Error, Request, Response, Server, StatusCode,
};
use lazy_static::lazy_static;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::Mutex;

use crate::payload::form_response;
use crate::webhook::{hash_parts, validate_request, WebhookBatch, WebhookPayload};

lazy_static! {
  static ref WEBHOOK_BATCHES: Mutex<HashMap<u64, WebhookBatch>> = Mutex::new(HashMap::new());
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

  let hashed = hash_parts(&webhook_parts);
  let payload: WebhookPayload = serde_json::from_str(&full_body).unwrap();
  let batch = payload::get_insert(hashed, webhook_parts, payload).await;
  let join = tokio::task::spawn(async move {
    let response = Response::builder()
      .status(StatusCode::OK)
      .header("X-Batch-Id", hashed)
      .header("X-Batch-Size", batch.payloads.len())
      .body(Body::empty())
      .unwrap();
    return response;
  });

  let response = join.await;
  return Ok(response.unwrap());
}

#[tokio::main]
async fn main() {
  let port: u16 = get_env("PORT", Some(8080));
  let addr = SocketAddr::from(([127, 0, 0, 1], port));
  let make_svc = make_service_fn(|_conn| async { Ok::<_, Error>(service_fn(handle_request)) });
  let server = Server::bind(&addr).serve(make_svc);
  println!("Server listening at {}", server.local_addr());
  if let Err(err) = server.await {
    eprintln!("Something went wrong: {}", err);
  }
}
