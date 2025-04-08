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
use kirby::clickhouse;
use kirby::s3::S3EventType;
use kirby::s3::read_object;
use lambda_runtime::Error;
use lambda_runtime::LambdaEvent;
use lambda_runtime::service_fn;
use lambda_runtime::tracing;
use lambda_runtime::tracing::info;
use lambda_runtime::tracing::warn;
use percent_encoding::percent_decode;

use std::env;

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

fn destination_key(key: &str, target_directory: &str) -> String {
    let parts = key
        .trim_start_matches("fastly_json/")
        .trim_end_matches(".log.gz")
        .split('/')
        .collect::<Vec<_>>();
    match parts[..] {
        [y, m, d, name] => format!("{}/{}/{}/{}/{}.json.gz", target_directory, y, m, d, name),
        [name] => {
            let y = &name[..4];
            let m = &name[5..7];
            let d = &name[8..10];
            format!("{}/{}/{}/{}/{}.json.gz", target_directory, y, m, d, name)
        }
        _ => format!("{}/{}.json.gz", target_directory, parts.join("/")),
    }
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
    let allow_backfill: bool = env::var("ALLOW_BACKFILL")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap();

    for record in event.payload.records {
        for record in record.sns.message.records {
            let target_directory = match record.event_name.as_ref().map(|s| s.parse()) {
                None => {
                    warn!("missing event name for record {:?}", record);
                    continue;
                }
                Some(Ok(S3EventType::ObjectRestoreCompleted)) => {
                    if !allow_backfill {
                        continue;
                    }
                    "backfill"
                }
                Some(Ok(ty)) if ty.is_object_created() => "incremental",
                _ => {
                    unreachable!("unexpected event type {:?}", record.event_name);
                }
            };
            let (bucket_name, url_key) = match (&record.s3.bucket.name, &record.s3.object.key) {
                (Some(bucket_name), Some(url_key)) => (bucket_name, url_key),
                _ => {
                    warn!("missing bucket name or key for record {:?}", record);
                    continue;
                }
            };

            let key = percent_decode(url_key.as_bytes()).decode_utf8()?;
            info!("downloading {}/{}", bucket_name, key);
            let reader = read_object(&s3_client, bucket_name, &key).await;

            info!("transforming entries...");
            let mut content = Vec::new();
            {
                let mut writer =
                    flate2::write::GzEncoder::new(&mut content, flate2::Compression::default());

                clickhouse(&mut writer, reader, &context)?;
            }
            let result_key = destination_key(key.as_ref(), target_directory);
            info!("uploading results to {}", &result_key);

            write_object(
                &gcp_client,
                &env::var("CLICKHOUSE_BUCKET").expect("CLICKHOUSE_BUCKET must be set"),
                &result_key,
                content,
            )
            .await;

            info!("done with {}", &key);
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

    #[test]
    fn test_destination_key() {
        let key = "fastly_json/2025/04/01/2025-04-01T04:00:00.000-W_DzA6b6s9QaCDzkOgDj.log.gz";
        let target_directory = "incremental";
        let expected =
            "incremental/2025/04/01/2025-04-01T04:00:00.000-W_DzA6b6s9QaCDzkOgDj.json.gz";
        assert_eq!(destination_key(key, target_directory), expected);

        let key = "fastly_json/2017-06-05T05:00:00.000-xpHgT4KL5P-M0PMAAAAA.log.gz";
        let expected =
            "incremental/2017/06/05/2017-06-05T05:00:00.000-xpHgT4KL5P-M0PMAAAAA.json.gz";
        assert_eq!(destination_key(key, target_directory), expected);

        let key = "fastly_json/2025/2025-04-01T04:00:00.000-W_DzA6b6s9QaCDzkOgDj.log.gz";
        assert_eq!(
            destination_key(key, target_directory),
            "incremental/2025/2025-04-01T04:00:00.000-W_DzA6b6s9QaCDzkOgDj.json.gz"
        );
    }
}
