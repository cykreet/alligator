use futures::TryStreamExt;
use hyper::Body;

const BODY_LIMIT: u16 = 32000; // 4KB

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

  // todo: check vec is empty, return error
  let something = full_body.map(|body| String::from_utf8(body).expect("Invalid UTF-8"));
  return Ok(something.unwrap());
}
