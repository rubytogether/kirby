extern crate flate2;
extern crate nom;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate time;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

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
use parsers::UserAgent;
use regex::Regex;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

fn read_file(filename: &str) -> Box<BufRead> {
  let path = Path::new(filename);
  let file = match File::open(&path) {
    Err(why) => panic!("couldn't open {}: {}", path.display(), why.description()),
    Ok(file) => file,
  };

  if path.extension() == Some(OsStr::new("gz")) {
    Box::new(BufReader::new(GzDecoder::new(file)))
  } else {
    Box::new(BufReader::new(file))
  }
}

fn user_agent_regex(a: &str) -> Option<UserAgent> {
  lazy_static!{
    static ref br: Regex = Regex::new(r"\Abundler/(?P<bundler>[0-9a-z.\-]+) rubygems/(?P<rubygems>[0-9a-z.\-]+) ruby/(?P<ruby>[0-9a-z.\-]+) \((?P<platform>.*)\) command/(?P<bundler_command>(.*?))( jruby/(?P<jruby>[0-9a-z.\-]+))?( truffleruby/(?P<truffleruby>[0-9a-z.\-]+))?( options/(?P<bundler_options>\S.*)?)?( ci/(?P<ci>\S.*)?)? (?P<bundler_command_uid>.*)( Gemstash/(?P<gemstash>[0-9a-z.\-]+))?\z").unwrap();
    static ref rr: Regex = Regex::new(r"\A(Ruby, )?RubyGems/(?P<rubygems>[0-9a-z.\-]+) (?P<platform>.*) Ruby/(?P<ruby>[0-9a-z.\-]+) \((?P<ruby_build>.*?)\)( jruby|truffleruby)?( Gemstash/(?P<gemstash>[0-9a-z.\-]+))?\z").unwrap();
  }

  let mut bl = br.capture_locations();
  let mut rl = rr.capture_locations();

  if let Some(ua) = br.captures_read(&mut bl, a) {
    return Some(UserAgent {
      bundler: match bl.get(1) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      rubygems: match bl.get(2) {
        Some(loc) => &a[loc.0..loc.1],
        _ => "",
      },
      ruby: match bl.get(3) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      platform: match bl.get(4) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      command: match bl.get(5) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      jruby: match bl.get(6) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      truffleruby: match bl.get(7) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      options: match bl.get(8) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      ci: match bl.get(9) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      uid: match bl.get(10) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      gemstash: match bl.get(11) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
    });
  } else {
    match rr.captures_read(&mut rl, a) {
      Some(_) => {
        return Some(UserAgent {
          bundler: None,
          rubygems: match rl.get(1) {
            Some(loc) => &a[loc.0..loc.1],
            _ => "",
          },
          ruby: match rl.get(3) {
            Some(loc) => Some(&a[loc.0..loc.1]),
            _ => None,
          },
          platform: match rl.get(2) {
            Some(loc) => Some(&a[loc.0..loc.1]),
            _ => None,
          },
          command: None,
          jruby: None,
          truffleruby: None,
          options: None,
          ci: None,
          uid: None,
          gemstash: match rl.get(5) {
            Some(loc) => Some(&a[loc.0..loc.1]),
            _ => None,
          },
        })
      }
      _ => None,
    }
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();

  let path = if args.len() > 1 {
    &args[1]
  } else {
    "test/sample_10.log.gz"
  };

  println!("going to open {}", path);
  let mut lineno = 0;
  let file = read_file(&path);

  for line in file.lines() {
    lineno += 1;
    if lineno % 100_000 == 0 {
      println!("{}", lineno);
    }

    let l = &line.unwrap();
    let r: Request = serde_json::from_str(l).unwrap();
    let t = time::strptime(&r.timestamp, "%F %T").unwrap();
    // println!("{}: {:?}", t.rfc3339(), user_agent);
    // if let Ok(ua) = parsers::user_agent(&r.user_agent) {
    // print!("{}: {:?}\n\n", t.rfc3339(), ua)
    // } else {
    // println!("{}: {}\n\n", t.rfc3339(), r.user_agent)
    // }
  }
}
