use nom::types::CompleteStr;
use nom::*;
// use std::str::FromStr;

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
      separated_nonempty_list!(tag!("."), alphanumeric)
    ))
  ))
);

complete_named!(bundler_version, do_parse!(tag!("bundler/") >> v: version >> (v)));
complete_named!(rubygems_version, do_parse!(alt!(tag!("Ruby, RubyGems/") | tag!("rubygems/")) >> v: version >> (v)));
complete_named!(ruby_version, do_parse!(alt!(tag!("ruby/") | tag!("Ruby/")) >> v: version >> (v)));
complete_named!(bundler_platform, delimited!(tag!("("), take_until!(")"), tag!(")")));
complete_named!(rubygems_platform, take_until!(" "));
complete_named!(command, do_parse!(tag!("command/") >> c: take_until!(" ") >> (c)));
complete_named!(options, do_parse!(tag!("options/") >> o: take_until!(" ") >> (o)));
complete_named!(uid, recognize!(hex_digit));

#[derive(PartialEq, Debug)]
pub struct UserAgent<'a> {
  bundler: Option<&'a str>,
  rubygems: &'a str,
  ruby: &'a str,
  platform: &'a str,
  command: Option<&'a str>,
  options: Option<&'a str>,
  uid: Option<&'a str>,
}

named!(
  bundler_user_agent<CompleteStr, UserAgent>,
  do_parse!(
      bundler: ws!(bundler_version) >>
      rubygems: ws!(rubygems_version) >>
      ruby: ws!(ruby_version) >>
      platform: ws!(bundler_platform) >>
      command: ws!(command) >>
      options: ws!(options) >>
      uid: ws!(uid) >>
      (UserAgent {
        bundler: Some(&bundler),
        rubygems: &rubygems,
        ruby: &ruby,
        platform: &platform,
        command: Some(&command),
        options: Some(&options),
        uid: Some(&uid)
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
      ruby: &ruby,
      platform: &platform,
      command: None,
      options: None,
      uid: None,
    })
  )
);

named!(any_user_agent<CompleteStr, UserAgent>,
  alt!(rubygems_user_agent | bundler_user_agent)
);

pub fn user_agent(s: &str) -> Result<UserAgent, Err<CompleteStr>> {
  match any_user_agent(CompleteStr(s)) {
    Result::Ok((_, ua)) => Result::Ok(ua),
    Result::Err(e) => Result::Err(e)
  }
}

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
    assert_complete(bundler_platform, "(x86_64-pc-linux-gnu)", "x86_64-pc-linux-gnu");
  }

  #[test]
  fn parse_rubygems_platform() {
    assert_eq!(
      rubygems_platform(CompleteStr("x86_64-linux Ruby/2.5.1")),
      Ok((CompleteStr(" Ruby/2.5.1"), CompleteStr("x86_64-linux")))
    );
  }

  fn assert_parsed(r: Result<(CompleteStr, UserAgent), Err<CompleteStr>>) {
    match r {
      Ok((_s, ua)) => assert!(true),
      Err(e) => panic!("didn't parse")
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
        ruby: "2.3.1",
        platform: "x86_64-pc-linux-gnu", 
        command: Some("install"), 
        options: Some("orig_path"), 
        uid: Some("95ac718b0e500f41")
      }
    );

    assert_parsed(bundler_user_agent(CompleteStr(
      "bundler/1.16.1 rubygems/2.6.11 ruby/2.4.1 (x86_64-pc-linux-gnu) command/install options/no_install,mirror.https://rubygems.org/,mirror.https://rubygems.org/.fallback_timeout/,path 59dbf8e99fa09c0a"
    )));
    assert_parsed(bundler_user_agent(CompleteStr(
      "bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41"
    )));
    assert_parsed(bundler_user_agent(CompleteStr(
      "bundler/1.16.1 rubygems/2.7.6 ruby/2.5.1 (x86_64-pc-linux-gnu) command/install options/no_install,git.allow_insecure,build.nokogiri,jobs,path,app_config,silence_root_warning,bin,gemfile e710485d04febb1e"
    )));
    assert_parsed(bundler_user_agent(CompleteStr(
      "bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41"
    )));
  }

  #[test]
  fn parse_rubygems_user_agent() {
    assert_eq!(
      rubygems_user_agent(CompleteStr(
        "Ruby, RubyGems/2.4.8 x86_64-linux Ruby/2.1.6 (2015-04-13 patchlevel 336)"
      )).unwrap().1,
      UserAgent { 
        bundler: None, 
        rubygems: "2.4.8",
        ruby: "2.1.6",
        platform: "x86_64-linux", 
        command: None, 
        options: None, 
        uid: None,
      }
    );

// Ruby, RubyGems/2.2.1 x86_64-linux Ruby/2.1.3 (2014-09-19 patchlevel 242)
// Ruby, RubyGems/2.2.5 x86_64-linux Ruby/2.1.8 (2016-04-22 patchlevel 492)
  }

  #[test]
  fn parse_user_agent() {
    assert_eq!(
      user_agent("bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41"),
      Ok(UserAgent { 
        bundler: Some("1.12.5"), 
        rubygems: "2.6.10",
        ruby: "2.3.1",
        platform: "x86_64-pc-linux-gnu", 
        command: Some("install"), 
        options: Some("orig_path"), 
        uid: Some("95ac718b0e500f41")
      })
    )
  }
}
