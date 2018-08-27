extern crate flate2;
extern crate nom;
extern crate regex;
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
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

fn read_file(path: &str) -> BufReader<GzDecoder<File>> {
  let path = Path::new(path);
  let file = match File::open(&path) {
    Err(why) => panic!("couldn't open {}: {}", path.display(), why.description()),
    Ok(file) => file,
  };

  BufReader::new(GzDecoder::new(file))
}

fn main() {
  let args: Vec<String> = env::args().collect();
  println!("{:?}", args);

  let path = if args.len() > 1 {
    &args[1]
  } else {
    "test/sample_10.log.gz"
  };

  println!("going to open {}", path);
  let mut lineno = 0;
  let file = read_file(&path);

  let bundler_matcher = Regex::new(r"\Abundler/(?P<bundler>[0-9a-z.\-]+) rubygems/(?P<rubygems>[0-9a-z.\-]+) ruby/(?P<ruby>[0-9a-z.\-]+) \((?P<platform>.*)\) command/(?P<bundler_command>(.*?))( jruby/(?P<jruby>[0-9a-z.\-]+))?( truffleruby/(?P<truffleruby>[0-9a-z.\-]+))?( options/(?P<bundler_options>\S.*)?)?( ci/(?P<ci>\S.*)?)? (?P<bundler_command_uid>.*)\z").unwrap();
  let rubygems_matcher = Regex::new(r"\A(Ruby, )?RubyGems/(?P<rubygems>[0-9a-z.\-]+) (?P<platform>.*) Ruby/(?P<ruby>[0-9a-z.\-]+) \((?P<ruby_build>.*?)\)( jruby|truffleruby)?( Gemstash/(?P<gemstash>[0-9a-z.\-]+))?\z").unwrap();

  for line in file.lines() {
    lineno += 1;
    if lineno % 100_000 == 0 {
      println!("{}", lineno);
    }

    let r: Request = serde_json::from_str(&line.unwrap()).unwrap();
    let t = time::strptime(&r.timestamp, "%F %T").unwrap();
    if let Some(cap) = rubygems_matcher.captures(&r.user_agent) {

    } else {
      let cap = bundler_matcher.captures(&r.user_agent);
    }
    // if let Ok(ua) = parsers::user_agent(&r.user_agent) {
    // print!("{}: {:?}\n\n", t.rfc3339(), ua)
    // } else {
    // println!("{}: {}\n\n", t.rfc3339(), r.user_agent)
    // }
  }
}
