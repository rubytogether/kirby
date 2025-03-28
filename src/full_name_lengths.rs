use std::{collections::HashMap, io::Read, sync::LazyLock};

use flate2::bufread::GzDecoder;
use lambda_runtime::tracing::info;

pub static FULL_NAME_LENGTHS_CSV: LazyLock<String> = LazyLock::new(|| {
    let mut buf = String::new();
    GzDecoder::new(include_bytes!("full_name_lengths.csv.gz").as_slice())
        .read_to_string(&mut buf)
        .unwrap();
    buf
});

pub static FULL_NAMES: LazyLock<HashMap<&'static str, (u8, u8)>> = LazyLock::new(|| {
    let mut full_names = HashMap::with_capacity(900_000);
    let mut lines = FULL_NAME_LENGTHS_CSV.lines();
    lines.next(); // skip header
    for line in lines {
        let mut parts = line.trim().split(',');
        let name = parts.next().expect("name");
        let name_length = parts.next().expect("name length").parse().unwrap();
        let version_length = parts.next().expect("version length").parse().unwrap();
        assert!(parts.remainder().is_none());
        full_names.insert(name, (name_length, version_length));
    }
    info!("{} full names loaded", full_names.len());
    full_names
});

#[test]
fn test_full_names() {
    assert!(!FULL_NAMES.is_empty());
}
