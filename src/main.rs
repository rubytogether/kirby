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

use argparse::{ArgumentParser, Collect, StoreTrue};
use std::collections::HashMap;
use std::io::*;

mod file;
mod request;
mod user_agent;

type StatsMap = HashMap<String, HashMap<String, HashMap<String, i32>>>;

fn file_to_stats(path: &str, opts: &Options) -> StatsMap {
  let mut lineno = 0;
  let mut times = HashMap::new();

  if opts.verbose {
    println!("Opening log file {}", path);
  }
  let file = file::reader(&path);

  for line in file.lines() {
    if opts.verbose {
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

    if opts.show_misses {
      match user_agent::parse(&r.user_agent) {
        None => println!("{}", r.user_agent),
        Some(_) => (),
      }
      continue;
    }

    if let Some(ua) = user_agent::parse(&r.user_agent) {
      {
        let rubygems = counters
          .entry("rubygems".to_string())
          .or_insert(HashMap::new());
        let count = rubygems.entry(String::from(ua.rubygems)).or_insert(0);
        *count += 1;
      }

      if let Some(bundler_version) = ua.bundler {
        let bundlers = counters
          .entry("bundler".to_string())
          .or_insert(HashMap::new());
        let count = bundlers.entry(String::from(bundler_version)).or_insert(0);
        *count += 1;
      }

      if let Some(ruby_version) = ua.ruby {
        let rubies = counters.entry("ruby".to_string()).or_insert(HashMap::new());
        let count = rubies.entry(String::from(ruby_version)).or_insert(0);
        *count += 1;
      }
    }
  }

  if opts.verbose {
    println!("");
  }

  times
}

struct Options {
  verbose: bool,
  show_misses: bool,
  paths: Vec<String>,
}

fn main() {
  let mut opts = Options {
    paths: ["test/sample_10.log.gz".to_string()].to_vec(),
    show_misses: false,
    verbose: false,
  };

  {
    let mut ap = ArgumentParser::new();
    ap.set_description("Parse a RubyGems.org Fastly JSON log file.");
    ap.refer(&mut opts.show_misses).add_option(
      &["--misses"],
      StoreTrue,
      "Print only user agents strings that failed to parse",
    );
    ap.refer(&mut opts.verbose)
      .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
    ap.refer(&mut opts.paths)
      .add_argument("FILE", Collect, "Path of the log file to parse");
    ap.parse_args_or_exit();
  }

  for path in &opts.paths {
    let times = file_to_stats(&path, &opts);
    println!("{:?}", times);
  }
}
