extern crate flate2;
extern crate nom;
extern crate serde;
extern crate serde_json;
extern crate time;

#[macro_use]
extern crate serde_derive;

mod parsers;

#[derive(Deserialize, Debug)]
struct Request {
  timestamp: String,
  // time_elapsed: u8,
  // client_ip: String,
  // client_continent: String,
  // client_country: String,
  // client_region: String,
  // client_city: String,
  // client_latitude: String,
  // client_longitude: String,
  client_timezone: String,
  // client_connection: String,
  // request: String,
  // request_host: String,
  request_path: String,
  // request_query: String,
  // request_bytes: u16,
  user_agent: String,
  http2: bool,
  tls: bool,
  tls_version: String,
  tls_cipher: String,
  // response_status: String,
  // response_text: String,
  // response_bytes: u16,
  // response_cache: String,
  // cache_state: String,
  // cache_lastuse: f32,
  // cache_hits: u16,
  // server_region: String,
  // server_datacenter: String,
}

use flate2::read::GzDecoder;
use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

fn read_file() -> BufReader<GzDecoder<File>> {
  // Create a path to the desired file
  let path = Path::new("test/sample_10.log.gz");
  let display = path.display();

  // Open the path in read-only mode, returns `io::Result<File>`
  let file = match File::open(&path) {
    // The `description` method of `io::Error` returns a string that
    // describes the error
    Err(why) => panic!("couldn't open {}: {}", display, why.description()),
    Ok(file) => file,
  };

  let d = GzDecoder::new(file);
  BufReader::new(d)
}

fn main() {
  let file = read_file();
  for line in file.lines() {
    let r: Request = serde_json::from_str(&line.unwrap()).unwrap();
    let t = time::strptime(&r.timestamp, "%F %T").unwrap();
    print!("{}: {:?}\n\n", t.rfc3339(), r.user_agent)
  }
}
