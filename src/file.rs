use flate2::read::GzDecoder;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

pub fn reader(filename: &str) -> Box<BufRead> {
  let path = Path::new(filename);
  let file = match File::open(&path) {
    Err(why) => panic!("couldn't open {}: {}", path.display(), why.description()),
    Ok(file) => file,
  };

  if path.extension() == Some(OsStr::new("gz")) {
    Box::new(BufReader::new(GzDecoder::new(file)))
  } else {
    Box::new(BufReader::new(file))
  }
}
