mod request;

pub use request::*;

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HttpVersion {
    major: u32,
    minor: u32,
}

impl FromStr for HttpVersion {
    type Err = ParseRequestErr;

    fn from_str(s: &str) -> Result<Self> {
        if !s.starts_with("HTTP/") {
            return Err(ParseRequestErr::InvalidVersion(s.to_string()));
        }

        let mut split = s[5..].split(".");
        let major: u32 = split
            .next().ok_or(ParseRequestErr::InvalidVersion(s.to_string()))?
            .parse()?;
        
        let minor: u32 = match split.next() {
            Some(v) => v.parse()?,
            None => 0,
        };

        Ok(Self { major, minor })
    }
}
