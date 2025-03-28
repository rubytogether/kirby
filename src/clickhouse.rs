use std::collections::HashMap;

use regex::Regex;

pub struct Context<'a> {
    pub full_name_lengths: &'a HashMap<&'a str, (u8, u8)>,
    pub download_pattern: Regex,
}

impl<'a> Context<'a> {
    pub fn new(full_name_lengths: &'a HashMap<&'a str, (u8, u8)>) -> Self {
        let download_pattern = Regex::new(r#"\A/gems/(.+)\.gem\z"#).unwrap();

        Context {
            full_name_lengths,
            download_pattern,
        }
    }
}
