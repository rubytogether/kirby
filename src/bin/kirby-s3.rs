#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

extern crate aws_lambda as lambda;
extern crate flate2;
extern crate futures;
extern crate kirby;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate time;

use flate2::read::GzDecoder;
use futures::stream::Stream;
use futures::Future;
use kirby::stream_stats;
use kirby::Options;
use crate::lambda::event::s3::S3Event;
use rusoto_core::region::Region;
use rusoto_s3::*;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Cursor;

fn read_object(bucket_name: &str, key: &str) -> Box<BufRead> {
  let get_req = GetObjectRequest {
    bucket: bucket_name.to_owned(),
    key: key.to_owned(),
    ..Default::default()
  };

  let client = S3Client::new(Region::UsWest2);
  let result = client
    .get_object(get_req)
    .sync()
    .expect("Couldn't GET object");

  let stream = result.body.unwrap();
  let bytes = stream
    .concat2()
    .wait()
    .expect("Couldn't read object body stream");

  if key.ends_with("gz") {
    Box::new(BufReader::new(GzDecoder::new(Cursor::new(bytes))))
  } else {
    Box::new(BufReader::new(Cursor::new(bytes)))
  }
}

fn write_object(bucket_name: &str, key: &str, body: &str) -> rusoto_s3::PutObjectOutput {
  let req = PutObjectRequest {
    bucket: bucket_name.to_owned(),
    key: key.to_owned(),
    body: Some(StreamingBody::from(body.as_bytes().to_vec())),
    ..Default::default()
  };

  let client = S3Client::new(Region::UsWest2);
  client.put_object(req).sync().expect("Couldn't PUT object")
}

fn main() {
  lambda::logger::init();

  lambda::start(|input: S3Event| {
    let opts = Options {
      paths: vec![],
      verbose: false,
      unknown: false,
    };

    for record in input.records {
      if let Some(bucket_name) = record.s3.bucket.name {
        if let Some(key) = record.s3.object.key {
          info!(
            "{} downloading {}/{}",
            time::now_utc().rfc3339(),
            &bucket_name,
            &key
          );
          let reader = read_object(&bucket_name, &key);

          info!("{} calculating stats...", time::now_utc().rfc3339());
          let content = stream_stats(reader, &opts);

          let result_key = [&key, ".json"]
            .concat()
            .replace("fastly_json", "fastly_stats");
          info!(
            "{} uploading results to {}",
            time::now_utc().rfc3339(),
            &result_key
          );
          write_object(&bucket_name, &result_key, &json!(content).to_string());

          info!("{} done with {}", time::now_utc().rfc3339(), &key);
        }
      }
    }

    Ok("ok")
  })
}
