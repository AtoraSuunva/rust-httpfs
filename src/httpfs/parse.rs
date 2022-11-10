use std::{collections::HashMap, str::FromStr};

use http::{header::HeaderName, HeaderValue, Request};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};

use crate::{httpfs::message::ByteRequest, httpfs::parse_error::HttpParseError};

pub async fn parse_request(stream: &mut TcpStream) -> Result<ByteRequest, HttpParseError> {
    let mut reader = BufReader::new(stream);
    let mut request = Request::builder();
    let mut content_length: usize = 0;

    // GET /path/to/file HTTP/1.1
    let mut status_line = String::new();
    reader.read_line(&mut status_line).await?;
    let mut status_line = status_line.trim_end().split_ascii_whitespace();

    let method = status_line
        .next()
        .ok_or_else(|| HttpParseError::MalformedRequest("Method missing".to_string()))?;

    let method = parse_method(method)?;

    let uri = status_line
        .next()
        .ok_or_else(|| HttpParseError::MalformedRequest("URI missing".to_string()))?;

    let version = status_line
        .next()
        .ok_or_else(|| HttpParseError::MalformedRequest("Version missing".to_string()))?;

    let version = parse_version(version)?;

    // Header-Name: header-value
    let headers = request.headers_mut().unwrap();

    loop {
        let mut header_line = String::new();
        if let 0 = reader.read_line(&mut header_line).await? {
            return Err(HttpParseError::EndOfStream);
        };

        if header_line == "\r\n" {
            break;
        }

        let (key, value) = parse_header(&header_line)?;

        if key == http::header::CONTENT_LENGTH {
            if method == http::Method::GET {
                return Err(HttpParseError::BodyNotAllowed);
            }

            content_length = value.to_str().unwrap().parse().map_err(|_| {
                HttpParseError::MalformedRequest("Content-Length is not a number".to_string())
            })?;
        }

        headers.append(key, value);
    }

    // \r\n
    // body bytes... (optional)
    let body = if content_length > 0 {
        let mut body = vec![0; content_length];
        reader.read_exact(&mut body).await?;
        Some(body)
    } else {
        None
    };

    let request = request
        .method(method)
        .uri(uri)
        .version(version)
        .body(body)?;

    Ok(request)
}

fn parse_method(str: &str) -> Result<http::Method, HttpParseError> {
    match http::Method::from_str(str) {
        Ok(method) => match method {
            http::Method::GET => Ok(method),
            http::Method::HEAD => Ok(method),
            http::Method::POST => Ok(method),
            http::Method::PUT => Ok(method),
            http::Method::DELETE => Ok(method),
            _ => Err(HttpParseError::UnsupportedMethod(str.to_string())),
        },
        Err(_) => Err(HttpParseError::UnsupportedMethod(str.to_string())),
    }
}

fn parse_version(str: &str) -> Result<http::Version, HttpParseError> {
    match str {
        "HTTP/1.0" => Ok(http::Version::HTTP_10),
        "HTTP/1.1" => Ok(http::Version::HTTP_11),
        "HTTP/2.0" => Ok(http::Version::HTTP_2),
        "HTTP/3.0" => Ok(http::Version::HTTP_3),
        _ => Err(HttpParseError::UnsupportedVersion(str.to_string())),
    }
}

fn parse_header(header_line: &str) -> Result<(HeaderName, HeaderValue), HttpParseError> {
    let header_line = header_line.trim_end();
    let mut header_parts = header_line.splitn(2, ": ");

    let key = header_parts
        .next()
        .ok_or_else(|| {
            HttpParseError::MalformedRequest(format!("Header name missing: '{}'", header_line))
        })?
        .parse::<HeaderName>()
        .map_err(|_| {
            HttpParseError::MalformedRequest(format!("Invalid header name: '{}'", header_line))
        })?;

    let value = header_parts
        .next()
        .ok_or_else(|| {
            HttpParseError::MalformedRequest(format!("Header value missing: '{}'", header_line))
        })?
        .parse::<HeaderValue>()
        .map_err(|_| {
            HttpParseError::MalformedRequest(format!("Invalid header value: '{}'", header_line))
        })?;

    Ok((key, value))
}

pub fn parse_query(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        let mut pair = pair.split('=');
        let key = pair.next().unwrap();
        let value = pair.next().unwrap();
        map.insert(key.to_string(), value.to_string());
    }
    map
}
