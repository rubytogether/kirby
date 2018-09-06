#![feature(test)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

extern crate argparse;
extern crate flate2;
extern crate nom;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate test;
extern crate time;

use argparse::{ArgumentParser, Store, StoreTrue};
use std::collections::HashMap;
use std::io::*;

mod file;
mod request;
mod user_agent;

fn main() {
  let mut verbose = false;
  let mut path = "test/sample_10.log.gz".to_string();
  let mut show_misses = false;

  {
    let mut ap = ArgumentParser::new();
    ap.set_description("Parse a RubyGems.org Fastly JSON log file.");
    ap.refer(&mut show_misses).add_option(
      &["--misses"],
      StoreTrue,
      "Print only user agents strings that failed to parse",
    );
    ap.refer(&mut verbose)
      .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
    ap.refer(&mut path)
      .add_argument("FILE", Store, "Path of the log file to parse");
    ap.parse_args_or_exit();
  }

  let mut lineno = 0;
  let mut times = HashMap::new();

  if verbose {
    println!("Opening log file {}", path);
  }
  let file = file::reader(&path);

  for line in file.lines() {
    if verbose {
      lineno += 1;
      if lineno % 100_000 == 0 {
        print!(".");
        stdout().flush().unwrap();
      }
    }

    let l = &line.unwrap();
    let r: request::Request = serde_json::from_str(l).unwrap();
    let hour = [r.timestamp.get(..14).unwrap(), "00:00"].concat();
    let mut counters = times.entry(hour).or_insert(HashMap::new());

    if show_misses {
      match user_agent::parse(&r.user_agent) {
        None => println!("{}", r.user_agent),
        Some(_) => (),
      }
      continue;
    }

    if let Some(ua) = user_agent::parse(&r.user_agent) {
      {
        let rubygems = counters.entry("rubygems").or_insert(HashMap::new());
        let count = rubygems.entry(String::from(ua.rubygems)).or_insert(0);
        *count += 1;
      }

      if let Some(bundler_version) = ua.bundler {
        let bundlers = counters.entry("bundler").or_insert(HashMap::new());
        let count = bundlers.entry(String::from(bundler_version)).or_insert(0);
        *count += 1;
      }

      if let Some(ruby_version) = ua.ruby {
        let rubies = counters.entry("ruby").or_insert(HashMap::new());
        let count = rubies.entry(String::from(ruby_version)).or_insert(0);
        *count += 1;
      }
    }
  }
  println!("");
  println!("{:?}", times);
}
