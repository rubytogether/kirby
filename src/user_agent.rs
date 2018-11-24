use regex::Regex;

#[derive(PartialEq, Debug)]
pub struct UserAgent<'a> {
  pub bundler: Option<&'a str>,
  pub rubygems: &'a str,
  pub ruby: Option<&'a str>,
  pub platform: Option<&'a str>,
  pub command: Option<&'a str>,
  pub options: Option<&'a str>,
  pub jruby: Option<&'a str>,
  pub truffleruby: Option<&'a str>,
  pub ci: Option<&'a str>,
  pub gemstash: Option<&'a str>,
}

pub fn parse(a: &str) -> Option<UserAgent> {
  lazy_static! {
    // Here is the named regex. The regex created below does not include names, because that interface has borrowing issues 😬
    // \Abundler/(?<bundler>[0-9a-zA-Z.\-]+) rubygems/(?<rubygems>[0-9a-zA-Z.\-]+) ruby/(?<ruby>[0-9a-zA-Z.\-]+) \((?<platform>.*)\) command/(.*?)(?: jruby/(?<jruby>[0-9a-zA-Z.\-]+))?(?: truffleruby/(?<truffleruby>[0-9a-zA-Z.\-]+))?(?: options/(?<options>.*?))?(?: ci/(?<ci>.*?))? ([a-f0-9]{16})(?: Gemstash/(?<gemstash>[0-9a-zA-Z.\-]+))?\z
    static ref br: Regex = Regex::new(r"\Abundler/([0-9a-zA-Z.\-]+) rubygems/([0-9a-zA-Z.\-]+) ruby/([0-9a-zA-Z.\-]+) \(([^)]*)\) command/(.*?)(?: jruby/([0-9a-zA-Z.\-]+))?(?: truffleruby/([0-9a-zA-Z.\-]+))?(?: options/(.*?))?(?: ci/(.*?))? [a-f0-9]{16}(?: Gemstash/([0-9a-zA-Z.\-]+))?\z").unwrap();
    static ref rr: Regex = Regex::new(r"\A(?:Ruby, )?RubyGems/([0-9a-z.\-]+) (.*) Ruby/([0-9a-z.\-]+) \(.*?\)(?: jruby| truffleruby| rbx)?(?: Gemstash/([0-9a-z.\-]+))?\z").unwrap();
    static ref gr: Regex = Regex::new(r"\ARuby, Gems ([0-9a-z.\-]+)\z").unwrap();
  }

  let mut bl = br.capture_locations();
  let mut rl = rr.capture_locations();
  let mut gl = gr.capture_locations();

  if let Some(_) = br.captures_read(&mut bl, a) {
    return Some(UserAgent {
      bundler: match bl.get(1) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
      rubygems: match bl.get(2) {
        Some(loc) => &a[loc.0..loc.1],
        _ => panic!("parse failed on {:?}", a),
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
      gemstash: match bl.get(11) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
    });
  } else if let Some(_) = rr.captures_read(&mut rl, a) {
    return Some(UserAgent {
      bundler: None,
      rubygems: match rl.get(1) {
        Some(loc) => &a[loc.0..loc.1],
        _ => panic!("parse failed on {:?}", a),
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
      gemstash: match rl.get(4) {
        Some(loc) => Some(&a[loc.0..loc.1]),
        _ => None,
      },
    });
  } else if let Some(_) = gr.captures_read(&mut gl, a) {
    return Some(UserAgent {
      bundler: None,
      rubygems: match gl.get(1) {
        Some(loc) => &a[loc.0..loc.1],
        _ => panic!("parse failed on {:?}", a),
      },
      ruby: None,
      platform: None,
      command: None,
      jruby: None,
      truffleruby: None,
      options: None,
      ci: None,
      gemstash: None,
    });
  } else {
    return None;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::file;
  use std::io::BufRead;
  use test::Bencher;
  use crate::user_agent::UserAgent;

  #[test]
  fn test_parse() {
    assert_eq!(
      parse("bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41"),
      Some(UserAgent {
        bundler: Some("1.12.5"),
        rubygems: "2.6.10",
        ruby: Some("2.3.1"),
        platform: Some("x86_64-pc-linux-gnu"),
        command: Some("install"),
        options: Some("orig_path"),
        jruby: None,
        truffleruby: None,
        ci: None,
        gemstash: None,
      })
    );

    assert_eq!(
      parse("Ruby, RubyGems/2.4.8 x86_64-linux Ruby/2.1.6 (2015-04-13 patchlevel 336)"),
      Some(UserAgent {
        bundler: None,
        rubygems: "2.4.8",
        ruby: Some("2.1.6"),
        platform: Some("x86_64-linux"),
        command: None,
        options: None,
        jruby: None,
        truffleruby: None,
        ci: None,
        gemstash: None,
      })
    );

    assert_eq!(
      parse("Ruby, Gems 1.1.1"),
      Some(UserAgent {
        bundler: None,
        rubygems: "1.1.1",
        ruby: None,
        platform: None,
        command: None,
        options: None,
        jruby: None,
        truffleruby: None,
        ci: None,
        gemstash: None,
      })
    );

    use crate::Options;
    let opts = Options {
      paths: vec![],
      verbose: false,
      unknown: false,
    };
    let file = file::reader("test/client_user_agents.txt", &opts);
    for line in file.lines() {
      let input = &line.unwrap();
      match parse(input) {
        Some(_) => assert!(true),
        None => panic!("couldn't parse {:?}", input),
      }
    }
  }

  #[bench]
  fn bench_parse(b: &mut Bencher) {
    b.iter(|| {
      parse("bundler/1.16.1 rubygems/2.6.11 ruby/2.4.1 (x86_64-pc-linux-gnu) command/install options/no_install,mirror.https://rubygems.org/,mirror.https://rubygems.org/.fallback_timeout/,path 59dbf8e99fa09c0a");
      parse("bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41");
      parse("bundler/1.16.1 rubygems/2.7.6 ruby/2.5.1 (x86_64-pc-linux-gnu) command/install rbx/3.105 options/no_install,git.allow_insecure,build.nokogiri,jobs,path,app_config,silence_root_warning,bin,gemfile e710485d04febb1e");
      parse("bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41");
      parse("bundler/1.15.4 rubygems/2.6.14 ruby/2.4.2 (x86_64-w64-mingw32) command/install options/ 6e8fa23dbf26d4ff Gemstash/1.1.0");
      parse("bundler/1.16.2 rubygems/2.7.6 ruby/2.5.0 (x86_64-Oracle Corporation-linux) command/install jruby/9.2.1.0-SNAPSHOT options/no_install,retry,jobs,gemfile ci/travis,ci fe5e45257d515f1f");
      parse("bundler/1.5.1 rubygems/2.2.0 ruby/2.1.0 (x86_64-unknown-linux-gnu) command/install fe5e45257d515f1f");
      parse("Ruby, Gems 1.1.1");
      parse("Ruby, RubyGems/1.3.7 x86_64-linux Ruby/1.9.2 (2010-08-18 patchlevel 0)");
      parse("Ruby, RubyGems/2.6.6 x86_64-linux Ruby/2.3.1 (2018-01-06 patchlevel 0) rbx");
    })
  }
}
