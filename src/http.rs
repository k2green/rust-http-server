use std::{collections::HashMap, iter::Peekable, str::{Chars, FromStr}};

#[derive(Debug, err_derive::Error)]
pub enum Error {
    #[error(display = "'{}' is not a valid HTTP method", _0)]
    InvalidMethod(String),
    #[error(display = "Failed to parse HTTP request head")]
    FailedToParseHead,
    #[error(display = "Failed to parse HTTP version")]
    FailedToParseVersion,
    #[error(display = "Parse int error: {}", _0)]
    ParseIntError(#[source] std::num::ParseIntError),
    #[error(display = "Failed to parse HTTP headers")]
    FailedToParseHeaders,
    #[error(display = "Failed to parse HTTP body")]
    FailedToParseBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FromStr for HttpMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "HEAD" => Ok(Self::HEAD),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            "PATCH" => Ok(Self::PATCH),
            _ => Err(Error::InvalidMethod(s.to_string()))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HttpVersion {
    major: usize,
    minor: usize
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    http_version: HttpVersion,
    headers: HashMap<String, String>,
    body: String,
}

impl FromStr for HttpRequest {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = RequestParser::from_str(s);
        parser.parse_request()
    }
}

struct RequestParser<Iter: Iterator<Item = char>> {
    characters: Peekable<Iter>
}

impl<'a> RequestParser<Chars<'a>> {
    fn from_str(input: &'a str) -> Self {
        Self { characters: input.chars().peekable() }
    }
}

impl<Iter: Iterator<Item = char>> RequestParser<Iter> {
    fn parse_request(&mut self) -> Result<HttpRequest, Error> {
        let method = self.parse_method()?;
        let path = self.parse_path()?;
        let http_version = self.parse_version()?;

        if !self.match_str("\r\n") {
            return Err(Error::FailedToParseHead);
        }
        
        let headers = self.parse_headers()?;
        let body = self.parse_body()?;

        Ok(HttpRequest { method, path, http_version, headers, body })
    }

    fn parse_method(&mut self) -> Result<HttpMethod, Error> {
        let mut method_str = String::new();
        while self.characters.peek().is_some_and(|c| c.is_ascii_uppercase()) {
            method_str.push(self.characters.next().unwrap());
        }

        self.skip_spaces();

        HttpMethod::from_str(&method_str)
    }

    fn parse_path(&mut self) -> Result<String, Error> {
        let mut path = String::new();
        while self.characters.peek().is_some_and(|c| *c != ' ') {
            path.push(self.characters.next().unwrap());
        }

        self.skip_spaces();

        Ok(path)
    }

    fn parse_version(&mut self) -> Result<HttpVersion, Error> {
        if !self.match_str("HTTP/") {
            return Err(Error::FailedToParseVersion);
        }

        let major = self.parse_usize()?;
        if !self.match_str(".") {
            return Err(Error::FailedToParseVersion);
        }

        let minor = self.parse_usize()?;

        Ok(HttpVersion { major, minor })
    }

    fn parse_headers(&mut self) -> Result<HashMap<String, String>, Error> {
        let mut headers = HashMap::new();
        while self.characters.peek().is_some_and(|c| *c != '\r' && *c != '\n') {
            let (key, val) = self.parse_header()?;
            headers.insert(key, val);
        }

        Ok(headers)
    }

    fn parse_header(&mut self) -> Result<(String, String), Error> {
        let mut header_key = String::new();
        while self.characters.peek().is_some_and(|c| *c != ':') {
            header_key.push(self.characters.next().unwrap());
        }

        if !self.match_str(": ") {
            return Err(Error::FailedToParseHeaders);
        }

        let mut header_value = String::new();
        while self.characters.peek().is_some_and(|c| *c != '\r' && *c != '\n') {
            header_value.push(self.characters.next().unwrap());
        }

        if !self.match_str("\r\n") {
            return Err(Error::FailedToParseHeaders);
        }

        Ok((header_key, header_value))
    }

    fn parse_body(&mut self) -> Result<String, Error> {
        if self.characters.peek().is_some() {
            if !self.match_str("\r\n") {
                return Err(Error::FailedToParseBody);
            }

            let mut body = String::new();
            while let Some(c) = self.characters.next() {
                body.push(c);
            }

            Ok(body)
        } else {
            Ok(String::new())
        }
    }

    fn skip_spaces(&mut self) {
        while self.characters.peek().is_some_and(|c| *c == ' ') {
            self.characters.next();
        }
    }

    fn parse_usize(&mut self) -> Result<usize, Error> {
        let mut digits = String::new();
        while self.characters.peek().is_some_and(|c| c.is_digit(10)) {
            digits.push(self.characters.next().unwrap());
        }

        usize::from_str_radix(&digits, 10).map_err(Error::from)
    }

    fn match_str(&mut self, input: &str) -> bool {
        for test_char in input.chars() {
            if self.characters.peek().is_none_or(|c| *c != test_char) {
                return false;
            }

            self.characters.next();
        }

        true
    }
}