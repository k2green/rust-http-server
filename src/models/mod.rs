mod request;
mod response;

pub use request::*;
pub use response::*;

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HttpVersion {
    major: u32,
    minor: u32,
}

impl HttpVersion {
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
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

impl std::fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP/{}", self.major)?;

        if self.minor > 0 {
            write!(f, ".{}", self.minor)?;
        }

        Ok(())
    }
}