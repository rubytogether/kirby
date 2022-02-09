#![feature(test)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate enum_map;

use enum_map::EnumMap;
use std::collections::HashMap;
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

#[allow(non_camel_case_types)]
#[derive(Debug, Enum, Serialize)]
pub enum FieldName {
  tls_cipher,
  server_region,
  rubygems,
  bundler,
  ruby,
  platform,
  ci,
  gemstash,
}

type ValueMap = HashMap<String, i32>;
type NameMap = EnumMap<FieldName, ValueMap>;
type TimeMap = HashMap<String, NameMap>;

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
      .or_insert(enum_map!{_ => HashMap::default()});
    for (name, versions) in names {
      let left_versions = &mut left_names[name];
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
    .or_insert(enum_map!{_ => HashMap::default()});

  increment(counters, FieldName::tls_cipher, r.tls_cipher.as_ref());

  if let Some(ua) = user_agent::parse(r.user_agent.as_ref()) {
    increment(counters, FieldName::rubygems, ua.rubygems);
    increment_maybe(counters, FieldName::bundler, ua.bundler);
    increment_maybe(counters, FieldName::ruby, ua.ruby);
    increment_maybe(counters, FieldName::platform, ua.platform);
    increment_maybe(counters, FieldName::ci, ua.ci);
    increment_maybe(counters, FieldName::gemstash, ua.gemstash);
  }
}

pub fn stream_stats(stream: Box<dyn BufRead>, opts: &Options) -> TimeMap {
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

#[inline]
pub fn file_stats(path: &str, opts: &Options) -> TimeMap {
  let file_stream = file::reader(&path, &opts);
  stream_stats(file_stream, &opts)
}
