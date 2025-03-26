#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

extern crate kirby;

use aws_lambda_events::event::s3::S3Event;
use flate2::read::GzDecoder;
use lambda_runtime::{Error, LambdaEvent, service_fn};
use percent_encoding::percent_decode;
use rusoto_core::region::Region;
use rusoto_s3::*;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;

use kirby::Options;
use kirby::stream_stats;

async fn read_object(bucket_name: &str, key: &str) -> Box<dyn BufRead> {
    let get_req = GetObjectRequest {
        bucket: bucket_name.to_owned(),
        key: key.to_owned(),
        ..Default::default()
    };

    let client = S3Client::new(Region::UsWest2);
    let result = client
        .get_object(get_req)
        .await
        .expect("Couldn't GET object");

    let mut bytes = Vec::new();
    result
        .body
        .unwrap()
        .into_blocking_read()
        .read_to_end(&mut bytes)
        .expect("Couldn't read object body stream");

    if key.ends_with("gz") {
        Box::new(BufReader::new(GzDecoder::new(Cursor::new(bytes))))
    } else {
        Box::new(BufReader::new(Cursor::new(bytes)))
    }
}

async fn write_object(bucket_name: &str, key: &str, body: &str) -> rusoto_s3::PutObjectOutput {
    let req = PutObjectRequest {
        bucket: bucket_name.to_owned(),
        key: key.to_owned(),
        body: Some(StreamingBody::from(body.as_bytes().to_vec())),
        ..Default::default()
    };

    let client = S3Client::new(Region::UsWest2);
    client.put_object(req).await.expect("Couldn't PUT object")
}

async fn func(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    let opts = Options {
        paths: vec![],
        verbose: false,
        unknown: false,
    };

    for record in event.payload.records {
        let (bucket_name, url_key) = match (&record.s3.bucket.name, &record.s3.object.key) {
            (Some(bucket_name), Some(url_key)) => (bucket_name, url_key),
            _ => {
                warn!("missing bucket name or key for record {:?}", record);
                continue;
            }
        };

        let key = percent_decode(url_key.as_bytes()).decode_utf8()?;
        info!(
            "{} downloading {}/{}",
            time::now_utc().rfc3339(),
            bucket_name,
            &key
        );
        let reader = read_object(bucket_name, &key).await;

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
        write_object(bucket_name, &result_key, &json!(content).to_string()).await;

        info!("{} done with {}", time::now_utc().rfc3339(), &key);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await
}
