use std::{
    io::{BufRead, BufReader, Cursor},
    str::FromStr,
};

use aws_sdk_s3::Client;
use flate2::bufread::GzDecoder;
use lambda_runtime::tracing::info;

pub async fn read_object(client: &Client, bucket_name: &str, key: &str) -> Box<dyn BufRead> {
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

    info!("read {} bytes for {}", bytes.len(), key);

    if key.ends_with("gz") {
        Box::new(BufReader::new(GzDecoder::new(Cursor::new(bytes))))
    } else {
        Box::new(BufReader::new(Cursor::new(bytes)))
    }
}

pub enum S3EventType {
    ObjectCreatedPut,
    ObjectCreatedPost,
    ObjectCreatedCopy,
    ObjectCreatedCompleteMultipartUpload,
    ObjectRestoreCompleted,
    Other(String),
}

impl S3EventType {
    pub fn is_object_created(&self) -> bool {
        matches!(
            self,
            S3EventType::ObjectCreatedPut
                | S3EventType::ObjectCreatedPost
                | S3EventType::ObjectCreatedCopy
                | S3EventType::ObjectCreatedCompleteMultipartUpload
        )
    }
}

impl FromStr for S3EventType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ObjectCreated:Put" => Ok(S3EventType::ObjectCreatedPut),
            "ObjectCreated:Post" => Ok(S3EventType::ObjectCreatedPost),
            "ObjectCreated:Copy" => Ok(S3EventType::ObjectCreatedCopy),
            "ObjectCreated:CompleteMultipartUpload" => {
                Ok(S3EventType::ObjectCreatedCompleteMultipartUpload)
            }
            "ObjectRestore:Completed" => Ok(S3EventType::ObjectRestoreCompleted),
            _ => Ok(S3EventType::Other(s.to_string())),
        }
    }
}
