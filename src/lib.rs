#![feature(test)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate enum_map;

use enum_map::EnumMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::IpAddr;
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

type UserIdentifier = IpAddr;

#[derive(Serialize)]
pub struct ValueUniqueCounter {
  total: usize,
  unique: usize,
  #[serde(skip_serializing)]
  index: HashSet<UserIdentifier>,
}

impl ValueUniqueCounter {
  fn new() -> ValueUniqueCounter {
    ValueUniqueCounter {
      total: 0,
      unique: 0,
      index: HashSet::new(),
    }
  }

  fn increment(&mut self, key: UserIdentifier) {
    self.total += 1;
    if self.index.insert(key) {
      self.unique += 1;
    }
  }

  fn combine(&mut self, other: &ValueUniqueCounter) {
    self.total += other.total;
    self.index = &self.index | &other.index;
    self.unique = self.index.len();
  }
}

impl Clone for ValueUniqueCounter {
  fn clone(&self) -> Self {
    ValueUniqueCounter {
      total: self.total,
      unique: self.unique,
      index: self.index.clone(),
    }
  }
}

type ValueMap = HashMap<String, ValueUniqueCounter>;
type NameMap = EnumMap<FieldName, ValueMap>;
type TimeMap = HashMap<String, NameMap>;

pub struct Options {
  pub verbose: bool,
  pub unknown: bool,
  pub paths: Vec<String>,
}

pub fn combine_stats(mut left: TimeMap, right: TimeMap) -> TimeMap {
  for (time, names) in right {
    let left_names = left
      .entry(time.to_string())
      .or_default();
    for (name, versions) in names {
      let left_versions = &mut left_names[name];
      for (version, counter) in versions {
        let left_counter = left_versions.entry(version.to_string()).or_insert(ValueUniqueCounter::new());
        left_counter.combine(&counter);
      }
    }
  }

  left
}

fn duplicate_request(r: &request::Request) -> bool {
  if r.request_path == "/api/v1/dependencies" {
    // Requests for dependencies are recursive, and so we want to count only one
    // request per time a user runs a command, rather than every request that was
    // made to satisfy that command. It seems like RubyGems makes one HEAD
    // request with no query, and Bundler makes one GET request with no query,
    // per command that is run. We ignore the rest for stats purposes.
    !r.request_query.is_empty()
  } else {
    // Versions that don't use the Dependency API make one request, either for
    // specs or for versions. We want to count each of those.
    !METADATA_PATHS.contains(&r.request_path.as_ref())
  }
}

fn increment(counters: &mut NameMap, name: FieldName, value: &str, key: UserIdentifier) {
  let counter = counters[name].entry(String::from(value)).or_insert(ValueUniqueCounter::new());
  counter.increment(key);
}

fn increment_maybe(counters: &mut NameMap, name: FieldName, maybe_value: Option<&str>, key: UserIdentifier) {
  if let Some(value) = maybe_value {
    increment(counters, name, value, key);
  }
}

pub fn print_unknown_user_agents(path: &str, opts: &Options) {
  file::reader(path, opts).split(b'\n').for_each(|line| {
    let l = &line.unwrap();
    let r: request::Request = serde_json::from_slice(l).unwrap();
    if user_agent::parse(r.user_agent.as_ref()).is_none() { println!("{}", r.user_agent) }
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

  let user_key = r.client_ip.parse().expect("ipaddr parse error");

  increment(counters, FieldName::tls_cipher, r.tls_cipher.as_ref(), user_key);
  if let Some(ua) = user_agent::parse(r.user_agent.as_ref()) {
    increment(counters, FieldName::rubygems, ua.rubygems, user_key);
    increment_maybe(counters, FieldName::bundler, ua.bundler, user_key);
    increment_maybe(counters, FieldName::ruby, ua.ruby, user_key);
    increment_maybe(counters, FieldName::platform, ua.platform, user_key);
    increment_maybe(counters, FieldName::ci, ua.ci, user_key);
    increment_maybe(counters, FieldName::gemstash, ua.gemstash, user_key);
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
    println!();
  }

  times
}

#[inline]
pub fn file_stats(path: &str, opts: &Options) -> TimeMap {
  let file_stream = file::reader(path, opts);
  stream_stats(file_stream, opts)
}
