#![feature(str_split_remainder)]

extern crate serde_json;

extern crate kirby;

use aws_credential_types::credential_fn::provide_credentials_fn;
use aws_lambda_events::s3::S3Event;
use aws_lambda_events::sns::SnsEventObj;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::operation::put_object::PutObjectOutput;
use aws_sdk_s3::primitives::ByteStream;
use flate2::read::GzDecoder;
use kirby::clickhouse;
use lambda_runtime::Error;
use lambda_runtime::LambdaEvent;
use lambda_runtime::service_fn;
use lambda_runtime::tracing;
use lambda_runtime::tracing::info;
use lambda_runtime::tracing::warn;
use percent_encoding::percent_decode;

use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Cursor;

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

    info!(
        "{} read {} bytes for {}",
        time::now_utc().rfc3339(),
        bytes.len(),
        key
    );

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
    let s3_client = {
        let config = aws_config::from_env().load().await;
        Client::new(&config)
    };

    let gcp_client = {
        let credentials_provider = provide_credentials_fn(|| async {
            let access_key_id = std::env::var("CLICKHOUSE_ACCESS_KEY_ID")
                .expect("CLICKHOUSE_ACCESS_KEY_ID must be set");
            let secret_access_key = std::env::var("CLICKHOUSE_SECRET_ACCESS_KEY")
                .expect("CLICKHOUSE_SECRET_ACCESS_KEY must be set");
            Ok(Credentials::new(
                access_key_id,
                secret_access_key,
                None,
                None,
                "clickhouse gcs",
            ))
        });
        let config = aws_config::from_env()
            .endpoint_url("https://storage.googleapis.com")
            .credentials_provider(credentials_provider)
            .load()
            .await;
        Client::new(&config)
    };

    let context = kirby::clickhouse::Context::new(&kirby::full_name_lengths::FULL_NAMES);

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
                key
            );
            let reader = read_object(&s3_client, bucket_name, &key).await;

            info!("{} transforming entries...", time::now_utc().rfc3339());
            let mut content = Vec::new();
            {
                let mut writer =
                    flate2::write::GzEncoder::new(&mut content, flate2::Compression::default());

                clickhouse(&mut writer, reader, &context)?;
            }
            let result_key = [
                key.replace("fastly_json", "incremental")
                    .trim_end_matches(".log.gz"),
                ".json.gz",
            ]
            .concat();
            info!(
                "{} uploading results to {}",
                time::now_utc().rfc3339(),
                &result_key
            );

            write_object(
                &gcp_client,
                &env::var("CLICKHOUSE_BUCKET").expect("CLICKHOUSE_BUCKET must be set"),
                &result_key,
                content,
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_func_empty_event() -> Result<(), Error> {
        let event = LambdaEvent {
            payload: SnsEventObj { records: vec![] },
            context: Default::default(),
        };

        func(event).await
    }
}
