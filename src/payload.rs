use hyper::Body;
use lazy_static::lazy_static;
use std::{
  collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
  },
  time::SystemTime,
};
use tokio::sync::Mutex;

use crate::webhook::{WebhookBatch, WebhookParts, WebhookPayload};

lazy_static! {
  static ref WEBHOOK_BATCHES: Mutex<HashMap<u64, WebhookBatch>> = Mutex::new(HashMap::new());
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

pub fn form_response(code: u8, message: &str) -> Body {
  return format!("{{ code: {}, message: {} }}", code, message).into();
}
