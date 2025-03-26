#[macro_use]
extern crate serde_json;

extern crate kirby;

use aws_config::meta::region::RegionProviderChain;
use aws_lambda_events::event::s3::S3Event;
use aws_lambda_events::sns::SnsEventObj;
use aws_sdk_s3::Client;
use aws_sdk_s3::operation::put_object::PutObjectOutput;
use aws_sdk_s3::primitives::ByteStream;
use flate2::read::GzDecoder;
use lambda_runtime::tracing::{self, info, warn};
use lambda_runtime::{Error, LambdaEvent, service_fn};
use percent_encoding::percent_decode;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Cursor;

use kirby::Options;
use kirby::stream_stats;

async fn read_object(client: &Client, bucket_name: &str, key: &str) -> Box<dyn BufRead> {
    let object = client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await
        .expect("Couldn't GET object");

    let bytes = object
        .body
        .collect()
        .await
        .expect("Couldn't collect object body stream")
        .into_bytes();

    if key.ends_with("gz") {
        Box::new(BufReader::new(GzDecoder::new(Cursor::new(bytes))))
    } else {
        Box::new(BufReader::new(Cursor::new(bytes)))
    }
}

async fn write_object<B>(client: &Client, bucket_name: &str, key: &str, body: B) -> PutObjectOutput
where
    B: Into<ByteStream>,
{
    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body.into())
        .send()
        .await
        .expect("Couldn't PUT object")
}

async fn func(event: LambdaEvent<SnsEventObj<S3Event>>) -> Result<(), Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let opts = Options {
        paths: vec![],
        verbose: false,
        unknown: false,
    };

    for record in event.payload.records {
        for record in record.sns.message.records {
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
            let reader = read_object(&client, bucket_name, &key).await;

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
            write_object(
                &client,
                bucket_name,
                &result_key,
                json!(content).to_string().into_bytes(),
            )
            .await;

            info!("{} done with {}", time::now_utc().rfc3339(), &key);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let func = service_fn(func);
    lambda_runtime::run(func).await
}
