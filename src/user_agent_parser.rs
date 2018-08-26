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
complete_named!(rubygems_version, do_parse!(tag!("rubygems/") >> v: version >> (v)));
complete_named!(ruby_version, do_parse!(tag!("ruby/") >> v: version >> (v)));
complete_named!(platform, delimited!(tag!("("), take_until!(")"), tag!(")")));
complete_named!(command, do_parse!(tag!("command/") >> c: take_until!(" ") >> (c)));
complete_named!(options, do_parse!(tag!("options/") >> o: take_until!(" ") >> (o)));
complete_named!(uid, recognize!(hex_digit));

#[derive(PartialEq, Debug)]
struct UserAgent<'a> {
  bundler: &'a str,
  rubygems: &'a str,
  ruby: &'a str,
  platform: &'a str,
  command: &'a str,
  options: &'a str,
  uid: &'a str,
}

named!(
  user_agent<CompleteStr, UserAgent>,
  do_parse!(
      bundler: ws!(bundler_version) >>
      rubygems: ws!(rubygems_version) >>
      ruby: ws!(ruby_version) >>
      platform: ws!(platform) >>
      command: ws!(command) >>
      options: ws!(options) >>
      uid: ws!(uid) >>
      (UserAgent {
        bundler: &bundler,
        rubygems: &rubygems,
        ruby: &ruby,
        platform: &platform,
        command: &command,
        options: &options,
        uid: &uid
      })
  )
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
  fn parse_platform() {
    assert_complete(platform, "(x86_64-pc-linux-gnu)", "x86_64-pc-linux-gnu");
  }

  #[test]
  fn parse_user_agent() {
    assert_eq!(
      user_agent(CompleteStr("bundler/1.12.5 rubygems/2.6.10 ruby/2.3.1 (x86_64-pc-linux-gnu) command/install options/orig_path 95ac718b0e500f41")).unwrap().1,
      UserAgent { 
        bundler: "1.12.5", 
        rubygems: "2.6.10",
        ruby: "2.3.1",
        platform: "x86_64-pc-linux-gnu", 
        command: "install", 
        options: "orig_path", 
        uid: "95ac718b0e500f41"
      }
    )
  }
}
