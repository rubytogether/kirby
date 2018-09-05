#![feature(test)]

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
extern crate test;

mod file;
mod parsers;
mod request;
mod user_agent;

use std::env;
use std::io::BufRead;
use user_agent::UserAgent;

fn main() {
  let args: Vec<String> = env::args().collect();

  let path = if args.len() > 1 {
    &args[1]
  } else {
    "test/sample_10.log.gz"
  };

  println!("going to open {}", path);
  let mut lineno = 0;
  let file = file::reader(&path);

  for line in file.lines() {
    lineno += 1;
    if lineno % 100_000 == 0 {
      println!("{}", lineno);
    }

    let l = &line.unwrap();
    let r: request::Request = serde_json::from_str(l).unwrap();
    let t = time::strptime(&r.timestamp, "%F %T").unwrap();

    if let Some(ua) = user_agent::parse(&r.user_agent) {
      print!("{} {} {:?}\n\n", t.rfc3339(), r.request_path, ua)
    } else {
      println!("{}", r.user_agent)
    }
  }
}
