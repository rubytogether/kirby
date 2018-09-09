#![feature(test)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate argparse;
extern crate flate2;
extern crate nom;
extern crate rayon;
extern crate regex;
extern crate serde;
extern crate test;
extern crate time;

use argparse::{ArgumentParser, Collect, StoreTrue};
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::*;

mod file;
mod request;
mod user_agent;

type ValueMap = HashMap<String, i32>;
type NameMap = HashMap<String, ValueMap>;
type TimeMap = HashMap<String, NameMap>;

fn combine_stats(left: &TimeMap, right: &TimeMap) -> TimeMap {
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

fn print_unknown_user_agents(path: &str, opts: &Options) {
  file::reader(&path, &opts).lines().for_each(|line| {
    let l = &line.unwrap();
    let r: request::Request = serde_json::from_str(l).unwrap();
    match user_agent::parse(&r.user_agent) {
      None => println!("{}", r.user_agent),
      Some(_) => (),
    }
  });
}

fn duplicate_request(r: &request::Request) -> bool {
  lazy_static! {
    static ref metadata_paths: Vec<String> = vec![
      "/latest_specs.4.8.gz".to_string(),
      "/prerelease_specs.4.8.gz".to_string(),
      "/specs.4.8.gz".to_string(),
      "/versions".to_string()
    ];
  }

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
    !metadata_paths.contains(&r.request_path)
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

fn file_to_stats(path: &str, opts: &Options) -> TimeMap {
  let mut lineno = 0;
  let mut times = HashMap::new();

  file::reader(&path, &opts).lines().for_each(|line| {
    if opts.verbose {
      lineno += 1;
      if lineno % 100_000 == 0 {
        print!(".");
        stdout().flush().unwrap();
      }
    }

    match line {
      Ok(l) => {
        let r: request::Request = serde_json::from_str(&l).unwrap();

        if duplicate_request(&r) {
          return;
        }

        let hour = [r.timestamp.get(..14).unwrap(), "00:00"].concat();
        let counters = times.entry(hour).or_insert(HashMap::new());

        increment(counters, "tls_cipher", &r.tls_cipher);
        increment(counters, "server_region", &r.server_region);

        if let Some(ua) = user_agent::parse(&r.user_agent) {
          increment(counters, "rubygems", ua.rubygems);
          increment_maybe(counters, "bundler", ua.bundler);
          increment_maybe(counters, "ruby", ua.ruby);
          increment_maybe(counters, "platform", ua.platform);
          increment_maybe(counters, "ci", ua.ci);
          increment_maybe(counters, "gemstash", ua.gemstash);
        }
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

pub struct Options {
  verbose: bool,
  unknown: bool,
  paths: Vec<String>,
}

fn main() {
  let mut opts = Options {
    paths: ["test/sample_10.log".to_string()].to_vec(),
    unknown: false,
    verbose: false,
  };

  {
    let mut ap = ArgumentParser::new();
    ap.set_description("Parse a RubyGems.org Fastly JSON log file.");
    ap.refer(&mut opts.unknown).add_option(
      &["-u", "--unknown"],
      StoreTrue,
      "Print only unrecognized user agent strings",
    );
    ap.refer(&mut opts.verbose)
      .add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
    ap.refer(&mut opts.paths)
      .add_argument("FILE", Collect, "Paths to the log file(s) to process");
    ap.parse_args_or_exit();
  }

  if opts.unknown {
    opts
      .paths
      .par_iter()
      .for_each(|path| print_unknown_user_agents(path, &opts));
    return;
  }

  let stats = opts
    .paths
    .par_iter()
    .map(|path| file_to_stats(&path, &opts))
    .reduce_with(|a, b| combine_stats(&a, &b))
    .unwrap();

  let output = json!({
    "ran_at": format!("{}", time::now_utc().rfc3339()),
    "stats": stats,
    "files": opts.paths,
  });
  println!("{}", output.to_string());
}
