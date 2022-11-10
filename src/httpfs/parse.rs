use std::{
    collections::HashMap,
    str::{from_utf8, FromStr},
};

use http::{header, header::HeaderName, HeaderValue, Method, Request, Version};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};

use crate::{httpfs::message::ByteRequest, httpfs::parse_error::HttpParseError};

pub async fn parse_request(stream: &mut TcpStream) -> Result<ByteRequest, HttpParseError> {
    let mut reader = BufReader::new(stream);
    let mut request = Request::builder();
    let mut content_length: usize = 0;
    let mut chunked = false;

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

        if key == header::CONTENT_LENGTH {
            if method == Method::GET {
                return Err(HttpParseError::BodyNotAllowed);
            }

            content_length = value.to_str().unwrap().parse().map_err(|_| {
                HttpParseError::MalformedRequest("Content-Length is not a number".to_string())
            })?;
        }

        if key == header::TRANSFER_ENCODING
            && value.to_str().unwrap().to_lowercase().contains("chunked")
        {
            chunked = true;
        }

        headers.append(key, value);
    }

    if method == Method::POST
        && !headers.contains_key(header::CONTENT_LENGTH)
        && !headers.contains_key(header::TRANSFER_ENCODING)
    {
        return Err(HttpParseError::LengthRequired);
    }

    // \r\n
    // body is optional, either
    //  - Content-Length: # body bytes...
    //  - Transfer-Encoding: chunked body...
    let body = if chunked {
        Some(read_chunked(reader).await?)
    } else if content_length > 0 {
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

async fn read_chunked(mut reader: BufReader<&mut TcpStream>) -> Result<Vec<u8>, HttpParseError> {
    let mut body = Vec::new();
    loop {
        // Read the chunk "head"
        // [hex octets]*(;ext-name=ext-val)\r\n
        // We need the num of octects in the chunk, but can ignore the chunk-ext
        // We don't recognize any chunk extensions, so we MUST ignore them

        // Read octets
        let mut octets: Vec<u8> = vec![];
        loop {
            let byte = reader.read_u8().await?;
            if byte == b';' || byte == b'\r' {
                break;
            }
            octets.push(byte);
        }

        // Read until end of line
        {
            let mut discard = vec![];
            reader.read_until(b'\n', &mut discard).await?;
        }

        let octets = usize::from_str_radix(from_utf8(&octets).unwrap(), 16)?;

        if octets == 0 {
            // We've reached the end of the chunked body
            break;
        }

        // Read the chunk
        for _ in 0..octets {
            body.push(reader.read_u8().await?);
        }

        // Read the chunk end
        {
            let mut discard = vec![];
            reader.read_until(b'\n', &mut discard).await?;
        }
    }

    Ok(body)
}

fn parse_method(str: &str) -> Result<Method, HttpParseError> {
    match Method::from_str(str) {
        Ok(method) => match method {
            Method::GET => Ok(method),
            Method::HEAD => Ok(method),
            Method::POST => Ok(method),
            Method::PUT => Ok(method),
            Method::DELETE => Ok(method),
            _ => Err(HttpParseError::UnsupportedMethod(str.to_string())),
        },
        Err(_) => Err(HttpParseError::UnsupportedMethod(str.to_string())),
    }
}

fn parse_version(str: &str) -> Result<Version, HttpParseError> {
    match str {
        "HTTP/1.0" => Ok(Version::HTTP_10),
        "HTTP/1.1" => Ok(Version::HTTP_11),
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

type QueryMap = HashMap<String, Option<String>>;

pub fn parse_query(query: &str) -> QueryMap {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        let mut pair = pair.split('=');
        let key = pair.next().unwrap();
        let value = pair.next().map(|f| f.to_string());

        map.insert(key.to_string(), value);
    }
    map
}
