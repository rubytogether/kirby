use nom::types::CompleteStr;
use nom::*;
use regex::Regex;

use user_agent::UserAgent;

#[macro_export]
macro_rules! complete_named (
  ($name:ident, $submac:ident!( $($args:tt)* )) => (
    fn $name( i: CompleteStr ) -> IResult<CompleteStr, CompleteStr, u32> {
      $submac!(i, $($args)*)
    }
  );
  ($name:ident<$o:ty>, $submac:ident!( $($args:tt)* )) => (
    fn $name( i: CompleteStr ) -> IResult<CompleteStr, $o, u32> {
      $submac!(i, $($args)*)
    }
  );
);

// '[0-9]+(?>\.[0-9a-zA-Z]+)*(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?'
complete_named!(
  version,
  recognize!(pair!(
    digit,
    opt!(pair!(
      tag!("."),
      separated_nonempty_list!(is_a!(".-"), alphanumeric)
    ))
  ))
);

complete_named!(
  bundler_version,
  do_parse!(tag!("bundler/") >> v: version >> (v))
);
complete_named!(
  rubygems_version,
  do_parse!(
    alt!(tag!("Ruby, RubyGems/") | tag!("RubyGems/") | tag!("rubygems/")) >> v: version >> (v)
  )
);
complete_named!(
  ruby_version,
  do_parse!(alt!(tag!("ruby/") | tag!("Ruby/")) >> v: version >> (v))
);
complete_named!(
  bundler_platform,
  delimited!(tag!("("), take_until!(")"), tag!(")"))
);
complete_named!(rubygems_platform, take_until!(" "));
complete_named!(
  jruby_version,
  do_parse!(tag!("jruby/") >> v: version >> (v))
);
complete_named!(
  options,
  do_parse!(tag!("options/") >> o: command_content >> (o))
);
complete_named!(ci, do_parse!(tag!("ci/") >> c: take_until!(" ") >> (c)));
complete_named!(gemstash, do_parse!(tag!("Gemstash/") >> v: version >> (v)));

complete_named!(
  command,
  do_parse!(tag!("command/") >> c: command_content >> (c))
);

fn command_content(input: CompleteStr) -> IResult<CompleteStr, CompleteStr> {
  let mut cmd = tag!(input, " ");

  match cmd {
    Ok(_) => return Ok((input, CompleteStr(""))),
    Err(_) => cmd = take_until!(input, " jruby/"),
  }

  match cmd {
    Ok(_) => return cmd,
    Err(_) => cmd = take_until!(input, " options/"),
  }

  match cmd {
    Ok(_) => return cmd,
    Err(_) => cmd = take_until!(input, " ci/"),
  }

  match cmd {
    Ok(_) => return cmd,
    Err(_) => {}
  }

  // Hopefully it's just a uid at the end
  let re = Regex::new(r"[0-9a-f]{16}$").unwrap();
  if re.is_match(&input) {
    return take!(input, (input.len() - 17));
  }

  // Sometimes it's a uid followed by a Gemstash version or the like
  let re = Regex::new(r"(.+)( [0-9a-f]{16} .+/.+)$").unwrap();
  let mut locs = re.capture_locations();
  let found = re.captures_read(&mut locs, &input);
  match found {
    Some(_) => {
      let l = locs.get(2).unwrap();
      let r = locs.get(1).unwrap();
      return Ok((CompleteStr(&input[l.0..l.1]), CompleteStr(&input[r.0..r.1])));
    }
    None => take!(input, input.len()),
  }
}

// uids need to be both all hex and exactly 16 characters long
fn uid(input: CompleteStr) -> IResult<CompleteStr, CompleteStr> {
  let rec = recognize!(input, hex_digit);
  match rec {
    Ok(hex) if hex.1.len() == 16 => rec,
    Ok(_) => Err(Err::Error(Context::Code(input, ErrorKind::TakeUntil))),
    Err(e) => Err(e),
  }
}

named!(
  bundler_user_agent<CompleteStr, UserAgent>,
  do_parse!(
      bundler: ws!(bundler_version) >>
      rubygems: ws!(rubygems_version) >>
      ruby: ws!(ruby_version) >>
      platform: ws!(bundler_platform) >>
      command: ws!(command) >>
      jruby: opt!(ws!(jruby_version)) >>
      options: opt!(ws!(options)) >>
      ci: opt!(ws!(ci)) >>
      uid: ws!(uid) >>
      gemstash: opt!(ws!(gemstash)) >>
      (UserAgent {
        bundler: Some(&bundler),
        rubygems: &rubygems,
        ruby: Some(&ruby),
        platform: Some(&platform),
        command: Some(&command),
        options: match options {
          Some(o) => Some(o.0),
          None => None
        },
        uid: Some(&uid),
        jruby: match jruby {
          Some(j) => Some(j.0),
          None => None
        },
        truffleruby: None,
        ci: match ci {
          Some(c) => Some(c.0),
          None => None,
        },
        gemstash: match gemstash {
          Some(g) => Some(g.0),
          None => None,
        }
      })
  )
);

// Ruby, RubyGems/2.4.8 x86_64-linux Ruby/2.1.6 (2015-04-13 patchlevel 336)
named!(rubygems_user_agent<CompleteStr, UserAgent>,
  do_parse!(
    rubygems: ws!(rubygems_version) >>
    platform: ws!(rubygems_platform) >>
    ruby: ws!(ruby_version) >>
    (UserAgent {
      bundler: None,
      rubygems: &rubygems,
      ruby: Some(&ruby),
      platform: Some(&platform),
      command: None,
      options: None,
      uid: None,
      jruby: None,
      truffleruby: None,
      ci: None,
      gemstash: None,
    })
  )
);

named!(old_rubygems_user_agent<CompleteStr, UserAgent>,
  do_parse!(
    tag!("Ruby, Gems ") >>
    rubygems: version >>
    (UserAgent {
      bundler: None,
      rubygems: &rubygems,
      ruby: None,
      platform: None,
      command: None,
      options: None,
      uid: None,
      jruby: None,
      truffleruby: None,
      ci: None,
      gemstash: None,
    })
  )
);

named!(pub any_user_agent<CompleteStr, UserAgent>,
  alt!(rubygems_user_agent | bundler_user_agent | old_rubygems_user_agent)
);

#[cfg(test)]
mod tests {
  use super::*;
  use nom::Context::Code;
  use nom::Err::Error;
  use nom::ErrorKind::*;
  use nom::IResult;

  fn assert_complete(
    f: fn(CompleteStr) -> IResult<CompleteStr, CompleteStr>,
    input: &str,
    res: &str,
  ) {
    assert_eq!(
      f(CompleteStr(input)),
      Ok((CompleteStr(""), CompleteStr(res)))
    )
  }

  #[test]
  fn parse_options() {
    assert_eq!(
      options(CompleteStr("options/ 95ac718b0e500f41")),
      Ok((CompleteStr(" 95ac718b0e500f41"), CompleteStr("")))
    );
    assert_eq!(
      options(CompleteStr("options/install 95ac718b0e500f41")),
      Ok((CompleteStr(" 95ac718b0e500f41"), CompleteStr("install")))
    );
    assert_eq!(
      options(CompleteStr("options/install ci/circle")),
      Ok((CompleteStr(" ci/circle"), CompleteStr("install")))
    );
    assert_eq!(
      options(CompleteStr("options/jobs, #git 95ac718b0e500f41")),
      Ok((CompleteStr(" 95ac718b0e500f41"), CompleteStr("jobs, #git")))
    );
    assert_eq!(
      options(CompleteStr("options/jobs, #git ci/circle 95ac718b0e500f41")),
      Ok((
        CompleteStr(" ci/circle 95ac718b0e500f41"),
        CompleteStr("jobs, #git")
      ))
    );
    assert_eq!(
      options(CompleteStr("options/ 6e8fa23dbf26d4ff Gemstash/1.1.0")),
      Ok((
        CompleteStr(" 6e8fa23dbf26d4ff Gemstash/1.1.0"),
        CompleteStr("")
      ))
    );
    assert_eq!(
      options(CompleteStr(
        "options/jobs, #git 95ac718b0e500f41 Gemstash/1.1.0"
      )),
      Ok((
        CompleteStr(" 95ac718b0e500f41 Gemstash/1.1.0"),
        CompleteStr("jobs, #git")
      ))
    );
  }

  #[test]
  fn parse_command() {
    assert_eq!(
      command(CompleteStr("command/install 95ac718b0e500f41")),
      Ok((CompleteStr(" 95ac718b0e500f41"), CompleteStr("install")))
    );
    assert_eq!(
      command(CompleteStr("command/install jruby/1")),
      Ok((CompleteStr(" jruby/1"), CompleteStr("install")))
    );
    assert_eq!(
      command(CompleteStr("command/install options/no")),
      Ok((CompleteStr(" options/no"), CompleteStr("install")))
    );
    assert_eq!(
      command(CompleteStr("command/--without test 95ac718b0e500f41")),
      Ok((
        CompleteStr(" 95ac718b0e500f41"),
        CompleteStr("--without test")
      ))
    );
    assert_eq!(
      command(CompleteStr(
        "command/--without test 95ac718b0e500f41 Gemstash/1.1.0"
      )),
      Ok((
        CompleteStr(" 95ac718b0e500f41 Gemstash/1.1.0"),
        CompleteStr("--without test")
      ))
    );
    assert_eq!(
      command(CompleteStr("command/--without test jruby/1 options/no")),
      Ok((
        CompleteStr(" jruby/1 options/no"),
        CompleteStr("--without test")
      ))
    );
    assert_eq!(
      command(CompleteStr("command/--without test options/no")),
      Ok((CompleteStr(" options/no"), CompleteStr("--without test")))
    );
  }

  #[test]
  fn parse_version() {
    assert_complete(version, "1.0", "1.0");
    assert_complete(version, "1", "1");
    assert_complete(version, "1.0a", "1.0a");
    assert_complete(version, "1.0.0", "1.0.0");
    assert_complete(version, "1.0.0a", "1.0.0a");
    assert_complete(
      version,
      "10000.1239857.12397858.12978362593487",
      "10000.1239857.12397858.12978362593487",
    );
    assert_eq!(
      version(CompleteStr("1a.1")),
      Ok((CompleteStr("a.1"), CompleteStr("1")))
    );
    assert_eq!(
      version(CompleteStr("a.1")),
      Err(Error(Code(CompleteStr("a.1"), Digit))),
    );
    assert_eq!(
      version(CompleteStr(".1")),
      Err(Error(Code(CompleteStr(".1"), Digit))),
    );
  }

  #[test]
  fn parse_bundler_version() {
    assert_complete(bundler_version, "bundler/1.16.2", "1.16.2");
    assert_eq!(
      bundler_version(CompleteStr("rubygems/1.16.2")),
      Err(Error(Code(CompleteStr("rubygems/1.16.2"), Tag)))
    );
  }

  #[test]
  fn parse_bundler_platform() {
    assert_complete(
      bundler_platform,
      "(x86_64-pc-linux-gnu)",
      "x86_64-pc-linux-gnu",
    );
  }

  #[test]
  fn parse_rubygems_platform() {
    assert_eq!(
      rubygems_platform(CompleteStr("x86_64-linux Ruby/2.5.1")),
      Ok((CompleteStr(" Ruby/2.5.1"), CompleteStr("x86_64-linux")))
    );
  }

  fn assert_parsed(f: fn(CompleteStr) -> IResult<CompleteStr, UserAgent>, input: &str) {
    let r = f(CompleteStr(input));
    match r {
      Ok((_s, ua)) => assert!(true),
      Err(e) => panic!("didn't parse {:?} with {}", input, e),
    }
  }

  #[test]
  fn parse_bundler_user_agent() {
    assert_eq!(
      bundler_user_agent(CompleteStr(
        "bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41"
      )).unwrap().1,
      UserAgent {
        bundler: Some("1.12.5"),
        rubygems: "2.6.10",
        ruby: Some("2.3.1"),
        platform: Some("x86_64-pc-linux-gnu"),
        command: Some("install"),
        options: Some("orig_path"),
        uid: Some("95ac718b0e500f41"),
        ci: None,
        gemstash: None,
        jruby: None,
        truffleruby: None
      }
    );

    assert_parsed(bundler_user_agent,
      "bundler/1.16.1 rubygems/2.6.11 ruby/2.4.1 (x86_64-pc-linux-gnu) command/install options/no_install,mirror.https://rubygems.org/,mirror.https://rubygems.org/.fallback_timeout/,path 59dbf8e99fa09c0a"
    );
    assert_parsed(bundler_user_agent,
      "bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41"
    );
    assert_parsed(bundler_user_agent,
      "bundler/1.16.1 rubygems/2.7.6 ruby/2.5.1 (x86_64-pc-linux-gnu) command/install options/no_install,git.allow_insecure,build.nokogiri,jobs,path,app_config,silence_root_warning,bin,gemfile e710485d04febb1e"
    );
    assert_parsed(bundler_user_agent,
      "bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41"
    );
    assert_parsed(bundler_user_agent,
      "bundler/1.15.4 rubygems/2.6.14 ruby/2.4.2 (x86_64-w64-mingw32) command/install options/ 6e8fa23dbf26d4ff Gemstash/1.1.0"
    );
    assert_parsed(bundler_user_agent,
      "bundler/1.16.2 rubygems/2.7.6 ruby/2.5.0 (x86_64-Oracle Corporation-linux) command/install jruby/9.2.1.0-SNAPSHOT options/no_install,retry,jobs,gemfile ci/travis,ci fe5e45257d515f1f"
    );
    assert_parsed(bundler_user_agent,
      "bundler/1.5.1 rubygems/2.2.0 ruby/2.1.0 (x86_64-unknown-linux-gnu) command/install fe5e45257d515f1f"
    );
  }

  #[test]
  fn parse_rubygems_user_agent() {
    assert_eq!(
      rubygems_user_agent(CompleteStr(
        "Ruby, RubyGems/2.4.8 x86_64-linux Ruby/2.1.6 (2015-04-13 patchlevel 336)"
      )).unwrap()
      .1,
      UserAgent {
        bundler: None,
        rubygems: "2.4.8",
        ruby: Some("2.1.6"),
        platform: Some("x86_64-linux"),
        command: None,
        options: None,
        uid: None,
        jruby: None,
        truffleruby: None,
        ci: None,
        gemstash: None,
      }
    );
  }

  #[test]
  fn parse_uid() {
    assert_complete(uid, "95ac718b0e500f41", "95ac718b0e500f41");
    assert_eq!(
      uid(CompleteStr("95ac718b0e500f411")),
      Err(Error(Code(CompleteStr("95ac718b0e500f411"), TakeUntil)))
    );
    assert_eq!(
      uid(CompleteStr("95ac718b0e500f4")),
      Err(Error(Code(CompleteStr("95ac718b0e500f4"), TakeUntil)))
    );
  }
}
