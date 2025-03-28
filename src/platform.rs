use core::str;
use std::{collections::VecDeque, slice};

use regex::{CaptureLocations, Regex};

#[derive(Debug)]
pub enum PlatformArgumentError {
    MissingCPU,
}

pub struct PlatformParser {
    i86_pattern: Regex,
    dotted_pattern: Regex,
    aix_pattern: Regex,
    darwin_pattern: Regex,
    freebsd_pattern: Regex,
    java_pattern: Regex,
    java_version_pattern: Regex,
    dalvik_pattern: Regex,
    dotnet_pattern: Regex,
    linux_pattern: Regex,
    mingw_pattern: Regex,
    mswin_pattern: Regex,
    openbsd_pattern: Regex,
    solaris_pattern: Regex,
}

impl PlatformParser {
    pub fn new() -> Self {
        Self {
            i86_pattern: Regex::new(r"i\d86").unwrap(),
            dotted_pattern: Regex::new(r"^\d+(?:\.\d+)?$").unwrap(),
            aix_pattern: Regex::new(r"aix(\d+)?").unwrap(),
            darwin_pattern: Regex::new(r"darwin(\d+)?").unwrap(),
            freebsd_pattern: Regex::new(r"freebsd(\d+)?").unwrap(),
            java_pattern: Regex::new(r"^(?:java|jruby)$").unwrap(),
            java_version_pattern: Regex::new(r"^java(\d+(?:\.\d+)*)?").unwrap(),
            dalvik_pattern: Regex::new(r"^dalvik(\d+)?$").unwrap(),
            dotnet_pattern: Regex::new(r"^dotnet(\d+(?:\.\d+)*)").unwrap(),
            linux_pattern: Regex::new(r"linux-?(\w+)?").unwrap(),
            mingw_pattern: Regex::new(r"mingw-?(\w+)?").unwrap(),
            mswin_pattern: Regex::new(r"(mswin\d+)(?:_(\d+))?").unwrap(),
            openbsd_pattern: Regex::new(r"openbsd(\d+\.\d+)?").unwrap(),
            solaris_pattern: Regex::new(r"solaris(\d+\.\d+)?").unwrap(),
        }
    }

    pub fn parse<'a>(&self, value: &'a str) -> Result<Platform<'a>, PlatformArgumentError> {
        let mut parts: VecDeque<_> = value.split('-').collect();

        // remove trailing empty string for consistency with ruby String#split
        while Some(&"") == parts.back() {
            parts.pop_back();
        }

        if parts.len() > 2 && !self.dotted_pattern.is_match(parts.back().unwrap()) {
            let extra = parts.pop_back().unwrap();
            let last = parts.pop_back().unwrap();
            let e = last.as_ptr();
            // SAFETY: we know that extra + last are from the same string, so the slice points to a single allocation
            // and is valid utf8
            let l = unsafe {
                str::from_utf8_unchecked(slice::from_raw_parts(e, extra.len() + last.len() + 1))
            };

            parts.push_back(l);
        }

        let mut cpu = match parts.pop_front() {
            Some(cpu) => {
                if cpu.is_empty() {
                    return Err(PlatformArgumentError::MissingCPU);
                }
                if self.i86_pattern.is_match(cpu) {
                    Some("x86")
                } else {
                    Some(cpu)
                }
            }
            None => return Err(PlatformArgumentError::MissingCPU),
        };

        if parts.len() == 2 && self.dotted_pattern.is_match(parts[1]) {
            return Ok(Platform {
                cpu,
                os: parts.pop_front().unwrap(),
                version: parts.pop_front(),
            });
        }

        let mut os = parts.pop_front().unwrap_or_else(|| {
            let os = cpu.unwrap();
            cpu = None;
            os
        });
        let mut version = None;

        let mut single_locs = self.aix_pattern.capture_locations();

        macro_rules! match_os {
            ($pattern:literal, $os:literal) => {
                if $pattern == os {
                    os = $os;
                    return Ok(Platform { cpu, os, version });
                }
            };
            (include $pattern:literal, $os:literal) => {
                if os.contains($pattern) {
                    os = $os;
                    return Ok(Platform { cpu, os, version });
                }
            };
            ($pattern:expr, $os:literal, 1) => {
                if $pattern.captures_read(&mut single_locs, &os).is_some() {
                    version = capture(&single_locs, os, 1);
                    os = $os;
                    return Ok(Platform { cpu, os, version });
                }
            };
            ($pattern:expr, $os:literal, 0) => {
                if $pattern.is_match(&os) {
                    os = $os;
                    return Ok(Platform { cpu, os, version });
                }
            };
        }

        match_os!(self.aix_pattern, "aix", 1);
        match_os!(include "cygwin", "cygwin");
        match_os!(self.darwin_pattern, "darwin", 1);
        match_os!("macruby", "macruby");
        match_os!(self.freebsd_pattern, "freebsd", 1);
        match_os!(self.java_pattern, "java", 0);
        match_os!(self.java_version_pattern, "java", 1);
        match_os!(self.dalvik_pattern, "dalvik", 1);
        match_os!("dotnet", "dotnet");
        match_os!(self.dotnet_pattern, "dotnet", 1);
        match_os!(self.linux_pattern, "linux", 1);
        match_os!(include "mingw32", "mingw32");
        match_os!(self.mingw_pattern, "mingw", 1);

        let mut double_locs = self.mswin_pattern.capture_locations();

        if self
            .mswin_pattern
            .captures_read(&mut double_locs, os)
            .is_some()
        {
            version = capture(&double_locs, os, 2);
            os = capture(&double_locs, os, 1).unwrap();
            if cpu.is_none() && os.ends_with("32") {
                cpu = Some("x86");
            }
            return Ok(Platform { cpu, os, version });
        }

        match_os!(include "netbsdelf", "netbsdelf");
        match_os!(self.openbsd_pattern, "openbsd", 1);
        match_os!(self.solaris_pattern, "solaris", 1);
        match_os!(include "wasi", "wasi");

        os = "unknown";

        Ok(Platform { cpu, os, version })
    }
}

#[derive(PartialEq, Debug, Eq, Serialize)]
pub struct Platform<'a> {
    cpu: Option<&'a str>,
    os: &'a str,
    version: Option<&'a str>,
}

impl std::fmt::Display for Platform<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let (None, os, Some(version)) = (&self.cpu, &self.os, &self.version) {
            return write!(f, "{}{}", os, version);
        }
        if let Some(cpu) = self.cpu {
            write!(f, "{}-", cpu)?;
        }

        write!(f, "{}", self.os)?;

        if let Some(version) = self.version {
            write!(f, "-{}", version)?;
        }
        Ok(())
    }
}

#[inline]
fn capture<'a>(loc: &CaptureLocations, s: &'a str, idx: usize) -> Option<&'a str> {
    let capture = loc.get(idx)?;
    Some(&s[capture.0..capture.1])
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_platform_from_str() {
        let tests: BTreeMap<&str, (Option<&str>, &str, Option<&str>)> = serde_json::from_str(
            r#"{
  "amd64-freebsd6": [
    "amd64",
    "freebsd",
    "6"
  ],
  "java": [
    null,
    "java",
    null
  ],
  "jruby": [
    null,
    "java",
    null
  ],
  "universal-dotnet": [
    "universal",
    "dotnet",
    null
  ],
  "universal-dotnet2.0": [
    "universal",
    "dotnet",
    "2.0"
  ],
  "universal-dotnet4.0": [
    "universal",
    "dotnet",
    "4.0"
  ],
  "powerpc-aix5.3.0.0": [
    "powerpc",
    "aix",
    "5"
  ],
  "powerpc-darwin7": [
    "powerpc",
    "darwin",
    "7"
  ],
  "powerpc-darwin8": [
    "powerpc",
    "darwin",
    "8"
  ],
  "powerpc-linux": [
    "powerpc",
    "linux",
    null
  ],
  "powerpc64-linux": [
    "powerpc64",
    "linux",
    null
  ],
  "sparc-solaris2.10": [
    "sparc",
    "solaris",
    "2.10"
  ],
  "sparc-solaris2.8": [
    "sparc",
    "solaris",
    "2.8"
  ],
  "sparc-solaris2.9": [
    "sparc",
    "solaris",
    "2.9"
  ],
  "universal-darwin8": [
    "universal",
    "darwin",
    "8"
  ],
  "universal-darwin9": [
    "universal",
    "darwin",
    "9"
  ],
  "universal-macruby": [
    "universal",
    "macruby",
    null
  ],
  "i386-cygwin": [
    "x86",
    "cygwin",
    null
  ],
  "i686-darwin": [
    "x86",
    "darwin",
    null
  ],
  "i686-darwin8.4.1": [
    "x86",
    "darwin",
    "8"
  ],
  "i386-freebsd4.11": [
    "x86",
    "freebsd",
    "4"
  ],
  "i386-freebsd5": [
    "x86",
    "freebsd",
    "5"
  ],
  "i386-freebsd6": [
    "x86",
    "freebsd",
    "6"
  ],
  "i386-freebsd7": [
    "x86",
    "freebsd",
    "7"
  ],
  "i386-freebsd": [
    "x86",
    "freebsd",
    null
  ],
  "universal-freebsd": [
    "universal",
    "freebsd",
    null
  ],
  "i386-java1.5": [
    "x86",
    "java",
    "1.5"
  ],
  "x86-java1.6": [
    "x86",
    "java",
    "1.6"
  ],
  "i386-java1.6": [
    "x86",
    "java",
    "1.6"
  ],
  "i686-linux": [
    "x86",
    "linux",
    null
  ],
  "i586-linux": [
    "x86",
    "linux",
    null
  ],
  "i486-linux": [
    "x86",
    "linux",
    null
  ],
  "i386-linux": [
    "x86",
    "linux",
    null
  ],
  "i586-linux-gnu": [
    "x86",
    "linux",
    "gnu"
  ],
  "i386-linux-gnu": [
    "x86",
    "linux",
    "gnu"
  ],
  "i386-mingw32": [
    "x86",
    "mingw32",
    null
  ],
  "x64-mingw-ucrt": [
    "x64",
    "mingw",
    "ucrt"
  ],
  "i386-mswin32": [
    "x86",
    "mswin32",
    null
  ],
  "i386-mswin32_80": [
    "x86",
    "mswin32",
    "80"
  ],
  "i386-mswin32-80": [
    "x86",
    "mswin32",
    "80"
  ],
  "x86-mswin32": [
    "x86",
    "mswin32",
    null
  ],
  "x86-mswin32_60": [
    "x86",
    "mswin32",
    "60"
  ],
  "x86-mswin32-60": [
    "x86",
    "mswin32",
    "60"
  ],
  "i386-netbsdelf": [
    "x86",
    "netbsdelf",
    null
  ],
  "i386-openbsd4.0": [
    "x86",
    "openbsd",
    "4.0"
  ],
  "i386-solaris2.10": [
    "x86",
    "solaris",
    "2.10"
  ],
  "i386-solaris2.8": [
    "x86",
    "solaris",
    "2.8"
  ],
  "mswin32": [
    "x86",
    "mswin32",
    null
  ],
  "x86_64-linux": [
    "x86_64",
    "linux",
    null
  ],
  "x86_64-linux-gnu": [
    "x86_64",
    "linux",
    "gnu"
  ],
  "x86_64-linux-musl": [
    "x86_64",
    "linux",
    "musl"
  ],
  "x86_64-linux-uclibc": [
    "x86_64",
    "linux",
    "uclibc"
  ],
  "arm-linux-eabi": [
    "arm",
    "linux",
    "eabi"
  ],
  "arm-linux-gnueabi": [
    "arm",
    "linux",
    "gnueabi"
  ],
  "arm-linux-musleabi": [
    "arm",
    "linux",
    "musleabi"
  ],
  "arm-linux-uclibceabi": [
    "arm",
    "linux",
    "uclibceabi"
  ],
  "x86_64-openbsd3.9": [
    "x86_64",
    "openbsd",
    "3.9"
  ],
  "x86_64-openbsd4.0": [
    "x86_64",
    "openbsd",
    "4.0"
  ],
  "x86_64-openbsd": [
    "x86_64",
    "openbsd",
    null
  ],
  "wasm32-wasi": [
    "wasm32",
    "wasi",
    null
  ],
  "wasm32-wasip1": [
    "wasm32",
    "wasi",
    null
  ],
  "wasm32-wasip2": [
    "wasm32",
    "wasi",
    null
  ],
  "darwin-java-java": [
    "darwin",
    "java",
    null
  ],
  "linux-linux-linux": [
    "linux",
    "linux",
    "linux"
  ],
  "linux-linux-linux1.0": [
    "linux",
    "linux",
    "linux1"
  ],
  "x86x86-1x86x86x86x861linuxx86x86": [
    "x86x86",
    "linux",
    "x86x86"
  ],
  "freebsd0": [
    null,
    "freebsd",
    "0"
  ],
  "darwin0": [
    null,
    "darwin",
    "0"
  ],
  "darwin0---": [
    null,
    "darwin",
    "0"
  ],
  "x86-linux-x8611.0l": [
    "x86",
    "linux",
    "x8611"
  ],
  "0-x86linuxx86---": [
    "0",
    "linux",
    "x86"
  ]
}
      "#,
        )
        .unwrap();
        let parser = PlatformParser::new();
        for (input, (cpu, os, version)) in tests {
            let expected = Platform { cpu, os, version };
            let platform = parser.parse(input).unwrap();
            assert_eq!(expected, platform, "{:?}", input);
            assert_eq!(
                expected,
                parser.parse(platform.to_string().as_str()).unwrap(),
                "{:?}: to_string {:?}",
                input,
                platform.to_string()
            );
        }
    }
}
