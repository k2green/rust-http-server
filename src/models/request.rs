use std::{collections::HashMap, fmt::Display, str::{FromStr, Lines}};

use err_derive::Error;

use super::HttpVersion;

pub type Result<T> = std::result::Result<T, ParseRequestErr>;

#[derive(Debug, Error)]
pub enum ParseRequestErr {
    #[error(display = "'{}' is not a valid http method", _0)]
    InvalidMethod(String),
    #[error(display = "'{}' is not a valid http version", _0)]
    InvalidVersion(String),
    #[error(display = "'{}' is not a valid http head", _0)]
    InvalidRequestHead(String),
    #[error(display = "'{}' is not a valid http header", _0)]
    InvalidHeader(String),
    #[error(display = "End of input reached unexpectedly")]
    UnexpectedEndOfInput,
    #[error(display = "Parse int error: {}", _0)]
    ParseIntError(#[source] std::num::ParseIntError),
    #[error(display = "From Utf8 error: {}", _0)]
    FromUtf8Error(#[source] std::string::FromUtf8Error),
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
    type Err = ParseRequestErr;

    fn from_str(s: &str) -> Result<Self> {
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
            _ => Err(ParseRequestErr::InvalidMethod(s.to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route(String);

impl Route {
    pub fn new(input: impl Display) -> Result<Self> {
        let input = input.to_string();
        let decoded = urlencoding::decode(&input)?;
        Ok(Self(decoded.into_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    method: HttpMethod,
    route: Route,
    version: HttpVersion,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpRequest {
    pub fn new(input: &str) -> Result<Self> {
        let mut lines = input.lines();
        let (method, route, version) = parse_head(&mut lines)?;
        let headers = parse_headers(&mut lines)?;
        let body = lines.collect::<Vec<_>>().join("\r\n");
        
        Ok(Self { method, route, version, headers, body })
    }
}

fn parse_head<'a>(lines: &mut Lines<'a>) -> Result<(HttpMethod, Route, HttpVersion)> {
    let head = lines.next()
        .ok_or(ParseRequestErr::UnexpectedEndOfInput)?;

    let mut split = head.split_whitespace();
    let method: HttpMethod = split
        .next()
        .ok_or(ParseRequestErr::InvalidRequestHead(head.to_string()))?
        .parse()?;

    let route =  Route::new(split
        .next()
        .ok_or(ParseRequestErr::InvalidRequestHead(head.to_string()))?)?;

    let version: HttpVersion =  split
        .next()
        .ok_or(ParseRequestErr::InvalidRequestHead(head.to_string()))?
        .parse()?;

    Ok((method, route, version))
}

fn parse_headers<'a>(lines: &mut Lines<'a>) -> Result<HashMap<String, String>> {
    let mut headers = HashMap::new();
    while let Some(line) = lines.next() {
        if line.trim().is_empty() { break; }
        
        let mut split = line.trim_start().split(": ");
        let key = split.next()
            .ok_or(ParseRequestErr::InvalidHeader(line.to_string()))?;

        let val = split.next()
            .ok_or(ParseRequestErr::InvalidHeader(line.to_string()))?;

        headers.insert(key.to_string(), val.to_string());
    }

    Ok(headers)
}