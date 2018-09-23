extern crate aws_lambda as lambda;
#[macro_use]
extern crate log;

use lambda::event::s3::S3Event;

fn main() {
  lambda::logger::init();

  lambda::start(|input: S3Event| {
    let mut files = Vec::new();
    for record in input.records {
      files.push((record.s3.bucket.name, record.s3.object.key));
    }

    info!("files {:#?}", files);
    Ok("done")
  })
}
