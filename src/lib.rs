#![feature(test)]
#![feature(extern_prelude)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

extern crate flate2;
extern crate nom;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate test;

use std::collections::HashMap;
use std::io::*;

mod file;
mod request;
mod user_agent;

const METADATA_PATHS: [&str; 4] = [
  "/latest_specs.4.8.gz",
  "/prerelease_specs.4.8.gz",
  "/specs.4.8.gz",
  "/versions"
];

type ValueMap = HashMap<String, i32>;
type NameMap = HashMap<String, ValueMap>;
type TimeMap = HashMap<String, NameMap>;

pub struct Options {
  pub verbose: bool,
  pub unknown: bool,
  pub paths: Vec<String>,
}

pub fn combine_stats(left: &TimeMap, right: &TimeMap) -> TimeMap {
  let mut left_times = left.clone();
  for (time, names) in right {
    let left_names = left_times.entry(time.to_string()).or_insert(HashMap::new());
    for (name, versions) in names {
      let left_versions = left_names.entry(name.to_string()).or_insert(HashMap::new());
      for (version, count) in versions {
        let left_count = left_versions.entry(version.to_string()).or_insert(0);
        *left_count += count;
      }
    }
  }

  left_times
}

fn duplicate_request(r: &request::Request) -> bool {
  if r.request_path == "/api/v1/dependencies" {
    // Requests for dependencies are recursive, and so we want to count only one
    // request per time a user runs a command, rather than every request that was
    // made to satisfy that command. It seems like RubyGems makes one HEAD
    // request with no query, and Bundler makes one GET request with no query,
    // per command that is run. We ignore the rest for stats purposes.
    r.request_query != ""
  } else {
    // Versions that don't use the Dependency API make one request, either for
    // specs or for versions. We want to count each of those.
    !METADATA_PATHS.contains(&r.request_path.as_ref())
  }
}

fn increment(counters: &mut NameMap, name: &str, value: &str) {
  let map = counters.entry(name.to_string()).or_insert(HashMap::new());
  let count = map.entry(String::from(value)).or_insert(0);
  *count += 1;
}

fn increment_maybe(counters: &mut NameMap, name: &str, maybe_value: Option<&str>) {
  if let Some(value) = maybe_value {
    increment(counters, name, value);
  }
}

pub fn print_unknown_user_agents(path: &str, opts: &Options) {
  file::reader(&path, &opts).lines().for_each(|line| {
    let l = &line.unwrap();
    let r: request::Request = serde_json::from_str(l).unwrap();
    match user_agent::parse(r.user_agent.as_ref()) {
      None => println!("{}", r.user_agent),
      Some(_) => (),
    }
  });
}

pub fn count_line(times: &mut TimeMap, line: String) {
  let r: request::Request = serde_json::from_str(&line).unwrap();

  if duplicate_request(&r) {
    return;
  }

  let date = r.timestamp.get(..10).unwrap().to_string();
  let counters = times.entry(date).or_insert(HashMap::new());

  increment(counters, "tls_cipher", r.tls_cipher.as_ref());
  increment(counters, "server_region", r.server_region.as_ref());

  if let Some(ua) = user_agent::parse(r.user_agent.as_ref()) {
    increment(counters, "rubygems", ua.rubygems);
    increment_maybe(counters, "bundler", ua.bundler);
    increment_maybe(counters, "ruby", ua.ruby);
    increment_maybe(counters, "platform", ua.platform);
    increment_maybe(counters, "ci", ua.ci);
    increment_maybe(counters, "gemstash", ua.gemstash);
  }
}

pub fn stream_stats(stream: Box<BufRead>, opts: &Options) -> TimeMap {
  let mut times = TimeMap::new();
  let mut lineno = 0;

  stream.lines().for_each(|line| {
    if opts.verbose {
      lineno += 1;
      if lineno % 100_000 == 0 {
        print!(".");
        stdout().flush().unwrap();
      }
    }

    match line {
      Ok(l) => {
        count_line(&mut times, l);
      }
      Err(e) => {
        if opts.verbose {
          eprintln!("Failed to read line:\n  {}", e);
        }
      }
    }
  });

  if opts.verbose {
    println!("");
  }

  times
}

pub fn file_stats(path: &str, opts: &Options) -> TimeMap {
  let file_stream = file::reader(&path, &opts);
  stream_stats(file_stream, &opts)
}
