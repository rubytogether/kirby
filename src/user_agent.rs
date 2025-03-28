use crate::platform::PlatformParser;
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Debug, Serialize, Default)]
pub struct UserAgent<'a> {
    pub agent_name: Option<&'a str>,
    pub agent_version: Option<&'a str>,
    pub bundler: Option<&'a str>,
    pub rubygems: Option<&'a str>,
    pub ruby: Option<&'a str>,
    #[serde(serialize_with = "serialize_platform")]
    pub platform: Option<&'a str>,
    pub command: Option<&'a str>,
    pub options: Option<&'a str>,
    pub jruby: Option<&'a str>,
    pub truffleruby: Option<&'a str>,
    pub ci: Option<&'a str>,
    pub gemstash: Option<&'a str>,
}

pub struct ParseCtx {
    bundler_pattern: Regex,
    ruby_pattern: Regex,
    gem_pattern: Regex,
    generic_pattern: Regex,
}

pub struct ParseCaptureLocations {
    bundler_captures: regex::CaptureLocations,
    ruby_captures: regex::CaptureLocations,
    gem_captures: regex::CaptureLocations,
    generic_captures: regex::CaptureLocations,
}

impl Default for ParseCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseCtx {
    /*
     * Here are some more regexes that indirect commented out so they didn't get moved to ParseCtx.
     * have fun :)
     */
    /*
    lazy_static! {
      // Here is the named regex. The regex created below does not include names, because that interface has borrowing issues 😬
      // \Abundler/(?<bundler>[0-9a-zA-Z.\-]+) rubygems/(?<rubygems>[0-9a-zA-Z.\-]+) ruby/(?<ruby>[0-9a-zA-Z.\-]+) \((?<platform>.*)\) command/(.*?)(?: jruby/(?<jruby>[0-9a-zA-Z.\-]+))?(?: truffleruby/(?<truffleruby>[0-9a-zA-Z.\-]+))?(?: options/(?<options>.*?))?(?: ci/(?<ci>.*?))? ([a-f0-9]{16})(?: Gemstash/(?<gemstash>[0-9a-zA-Z.\-]+))?\z
    }
    */
    pub fn new() -> Self {
        Self {
            bundler_pattern: Regex::new(r"\Abundler/([0-9a-zA-Z.\-]+) rubygems/([0-9a-zA-Z.\-]+) ruby/([0-9a-zA-Z.\-]+) \(([^)]*)\) command/(.*?)(?: jruby/([0-9a-zA-Z.\-]+))?(?: truffleruby/([0-9a-zA-Z.\-]+))?(?: options/(.*?))?(?: ci/(.*?))? [a-f0-9]{16}(?: Gemstash/([0-9a-zA-Z.\-]+))?\z").unwrap(),
            ruby_pattern: Regex::new(r"\A(?:Ruby, )?RubyGems/([0-9a-z.\-]+) (.*) Ruby/([0-9a-z.\-]+) \(.*?\)(?: jruby| truffleruby| rbx)?(?: Gemstash/([0-9a-z.\-]+))?\z").unwrap(),
            // From the "gems" gem
            gem_pattern: Regex::new(r"\ARuby, Gems ([0-9a-z.\-]+)\z").unwrap(),
            generic_pattern: Regex::new(r"\A(?:Ruby, )?([^ /]+)(?:/([^ /]+))?").unwrap(),
        }
    }

    pub fn capture_locations(&self) -> ParseCaptureLocations {
        ParseCaptureLocations {
            bundler_captures: self.bundler_pattern.capture_locations(),
            ruby_captures: self.ruby_pattern.capture_locations(),
            gem_captures: self.gem_pattern.capture_locations(),
            generic_captures: self.generic_pattern.capture_locations(),
        }
    }

    pub fn parse<'line>(
        &self,
        capture_locations: &mut ParseCaptureLocations,
        a: &'line str,
    ) -> Option<UserAgent<'line>> {
        let bl = &mut capture_locations.bundler_captures;
        let rl = &mut capture_locations.ruby_captures;
        let gl = &mut capture_locations.gem_captures;

        if self.bundler_pattern.captures_read(bl, a).is_some() {
            Some(UserAgent {
                agent_name: Some("bundler"),
                agent_version: match bl.get(1) {
                    Some(loc) => Some(&a[loc.0..loc.1]),
                    _ => None,
                },
                bundler: match bl.get(1) {
                    Some(loc) => Some(&a[loc.0..loc.1]),
                    _ => None,
                },
                rubygems: match bl.get(2) {
                    Some(loc) => Some(&a[loc.0..loc.1]),
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
            })
        } else if self.ruby_pattern.captures_read(rl, a).is_some() {
            return Some(UserAgent {
                agent_name: Some("rubygems"),
                agent_version: match rl.get(1) {
                    Some(loc) => Some(&a[loc.0..loc.1]),
                    _ => panic!("parse failed on {:?}", a),
                },
                bundler: None,
                rubygems: match rl.get(1) {
                    Some(loc) => Some(&a[loc.0..loc.1]),
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
        } else if self.gem_pattern.captures_read(gl, a).is_some() {
            return Some(UserAgent {
                agent_name: Some("gems"),
                agent_version: match gl.get(1) {
                    Some(loc) => Some(&a[loc.0..loc.1]),
                    _ => panic!("parse failed on {:?}", a),
                },
                bundler: None,
                rubygems: None,
                ruby: None,
                platform: None,
                command: None,
                jruby: None,
                truffleruby: None,
                options: None,
                ci: None,
                gemstash: None,
            });
        } else if self
            .generic_pattern
            .captures_read(&mut capture_locations.generic_captures, a)
            .is_some()
        {
            return Some(UserAgent {
                agent_name: capture_locations
                    .generic_captures
                    .get(1)
                    .map(|m| &a[m.0..m.1]),
                agent_version: capture_locations
                    .generic_captures
                    .get(2)
                    .map(|m| &a[m.0..m.1]),
                ..Default::default()
            });
        } else {
            return None;
        }
    }
}

pub fn serialize_user_agent<S>(ua: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    lazy_static! {
        static ref USER_AGENT_PARSER: ParseCtx = ParseCtx::new();
    };
    let mut capture_locations = USER_AGENT_PARSER.capture_locations();
    let user_agent = USER_AGENT_PARSER.parse(&mut capture_locations, ua);
    serde::Serialize::serialize(&user_agent, serializer)
}

pub fn serialize_platform<S>(pl: &Option<&str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    lazy_static! {
        static ref PLATFORM_PARSER: PlatformParser = PlatformParser::new();
    };

    match pl {
        None => serde::Serialize::serialize(pl, serializer),
        Some(pl) => {
            let platform = PLATFORM_PARSER
                .parse(pl)
                .inspect_err(|_| info!("unable to parse platform {:?}", pl))
                .ok();
            serde::Serialize::serialize(&platform, serializer)
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use crate::file;
    use crate::user_agent::UserAgent;
    use std::io::BufRead;
    use test::Bencher;

    #[test]
    fn test_parse() {
        let ctx = ParseCtx::new();
        let mut capture_locations = ctx.capture_locations();
        assert_eq!(
            ctx.parse(
                &mut capture_locations,
                "bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41"
            ),
            Some(UserAgent {
                agent_name: Some("bundler"),
                agent_version: Some("1.12.5"),
                bundler: Some("1.12.5"),
                rubygems: Some("2.6.10"),
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
            ctx.parse(
                &mut capture_locations,
                "Ruby, RubyGems/2.4.8 x86_64-linux Ruby/2.1.6 (2015-04-13 patchlevel 336)"
            ),
            Some(UserAgent {
                agent_name: Some("rubygems"),
                agent_version: Some("2.4.8"),
                bundler: None,
                rubygems: Some("2.4.8"),
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
            ctx.parse(&mut capture_locations, "Ruby, Gems 1.1.1"),
            Some(UserAgent {
                agent_name: Some("gems"),
                agent_version: Some("1.1.1"),
                bundler: None,
                rubygems: None,
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
            ctx.parse(&mut capture_locations, input)
                .unwrap_or_else(|| panic!("couldn't parse {:?}", input));
        }
    }

    #[bench]
    fn bench_parse(b: &mut Bencher) {
        let ctx = ParseCtx::new();
        let mut capture_locations = ctx.capture_locations();
        b.iter(|| {
            ctx.parse(&mut capture_locations,"bundler/1.16.1 rubygems/2.6.11 ruby/2.4.1 (x86_64-pc-linux-gnu) command/install options/no_install,mirror.https://rubygems.org/,mirror.https://rubygems.org/.fallback_timeout/,path 59dbf8e99fa09c0a");
            ctx.parse(&mut capture_locations,"bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41");
            ctx.parse(&mut capture_locations,"bundler/1.16.1 rubygems/2.7.6 ruby/2.5.1 (x86_64-pc-linux-gnu) command/install rbx/3.105 options/no_install,git.allow_insecure,build.nokogiri,jobs,path,app_config,silence_root_warning,bin,gemfile e710485d04febb1e");
            ctx.parse(&mut capture_locations,"bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41");
            ctx.parse(&mut capture_locations,"bundler/1.15.4 rubygems/2.6.14 ruby/2.4.2 (x86_64-w64-mingw32) command/install options/ 6e8fa23dbf26d4ff Gemstash/1.1.0");
            ctx.parse(&mut capture_locations,"bundler/1.16.2 rubygems/2.7.6 ruby/2.5.0 (x86_64-Oracle Corporation-linux) command/install jruby/9.2.1.0-SNAPSHOT options/no_install,retry,jobs,gemfile ci/travis,ci fe5e45257d515f1f");
            ctx.parse(&mut capture_locations,"bundler/1.5.1 rubygems/2.2.0 ruby/2.1.0 (x86_64-unknown-linux-gnu) command/install fe5e45257d515f1f");
            ctx.parse(&mut capture_locations,"Ruby, Gems 1.1.1");
            ctx.parse(&mut capture_locations,"Ruby, RubyGems/1.3.7 x86_64-linux Ruby/1.9.2 (2010-08-18 patchlevel 0)");
            ctx.parse(&mut capture_locations,"Ruby, RubyGems/2.6.6 x86_64-linux Ruby/2.3.1 (2018-01-06 patchlevel 0) rbx");
        })
    }
}
