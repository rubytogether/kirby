#![feature(test)]
#![feature(extern_prelude)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate enum_map;
extern crate flate2;
extern crate fnv;
extern crate nom;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate test;

use enum_map::EnumMap;
use fnv::FnvHashMap;
use std::io::*;

mod file;
mod request;
mod user_agent;

const METADATA_PATHS: [&str; 4] = [
  "/latest_specs.4.8.gz",
  "/prerelease_specs.4.8.gz",
  "/specs.4.8.gz",
  "/versions",
];

#[derive(Debug, Enum)]
pub enum FieldName {
  TlsCipher,
  ServerRegion,
  Rubygems,
  Bundler,
  Ruby,
  Platform,
  Ci,
  Gemstash,
}

type ValueMap = FnvHashMap<String, i32>;
type NameMap = EnumMap<FieldName, ValueMap>;
type TimeMap = FnvHashMap<String, NameMap>;

pub struct Options {
  pub verbose: bool,
  pub unknown: bool,
  pub paths: Vec<String>,
}

pub fn combine_stats(left: &TimeMap, right: &TimeMap) -> TimeMap {
  let mut left_times = left.clone();
  for (time, names) in right {
    let left_names = left_times
      .entry(time.to_string())
      .or_insert(enum_map!{_ => FnvHashMap::default()});
    for (name, versions) in names {
      let mut left_versions = &mut left_names[name];
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

fn increment(counters: &mut NameMap, name: FieldName, value: &str) {
  let count = counters[name].entry(String::from(value)).or_insert(0);
  *count += 1;
}

fn increment_maybe(counters: &mut NameMap, name: FieldName, maybe_value: Option<&str>) {
  if let Some(value) = maybe_value {
    increment(counters, name, value);
  }
}

pub fn print_unknown_user_agents(path: &str, opts: &Options) {
  file::reader(&path, &opts).split(b'\n').for_each(|line| {
    let l = &line.unwrap();
    let r: request::Request = serde_json::from_slice(l).unwrap();
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
  let counters = times
    .entry(date)
    .or_insert(enum_map!{_ => FnvHashMap::default()});

  increment(counters, FieldName::TlsCipher, r.tls_cipher.as_ref());

  if let Some(ua) = user_agent::parse(r.user_agent.as_ref()) {
    increment(counters, FieldName::Rubygems, ua.rubygems);
    increment_maybe(counters, FieldName::Bundler, ua.bundler);
    increment_maybe(counters, FieldName::Ruby, ua.ruby);
    increment_maybe(counters, FieldName::Platform, ua.platform);
    increment_maybe(counters, FieldName::Ci, ua.ci);
    increment_maybe(counters, FieldName::Gemstash, ua.gemstash);
  }
}

pub fn stream_stats(stream: Box<BufRead>, opts: &Options) -> TimeMap {
  let mut times = TimeMap::default();
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
