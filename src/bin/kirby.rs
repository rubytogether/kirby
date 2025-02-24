extern crate argparse;
#[macro_use]
extern crate serde_json;
extern crate kirby;
extern crate rayon;
extern crate time;

use argparse::{ArgumentParser, Collect, StoreTrue};
use kirby::Options;
use rayon::prelude::*;

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
      .for_each(|path| kirby::print_unknown_user_agents(path, &opts));
    return;
  }

  let stats = opts
    .paths
    .par_iter()
    .map(|path| kirby::file_stats(path, &opts))
    .reduce_with(|a, b| kirby::combine_stats(&a, &b))
    .unwrap();

  let output = json!({
    "ran_at": format!("{}", time::now_utc().rfc3339()),
    "stats": stats,
    "files": opts.paths,
  });
  println!("{}", output);
}
