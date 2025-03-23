use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

pub fn reader(filename: &str, opts: &super::Options) -> Box<dyn BufRead> {
  if opts.verbose {
    println!("Opening log file {}", filename);
  }

  let path = Path::new(filename);
  let file = match File::open(path) {
    Err(why) => panic!("couldn't open {}: {}", path.display(), why),
    Ok(file) => file,
  };

  if path.extension() == Some(OsStr::new("gz")) {
    Box::new(BufReader::with_capacity(128 * 1024, GzDecoder::new(file)))
  } else {
    Box::new(BufReader::with_capacity(128 * 1024, file))
  }
}
