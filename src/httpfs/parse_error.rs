use std::{num::ParseIntError, string::FromUtf8Error};

use http::StatusCode;

#[derive(Debug, Clone)]
pub enum HttpParseError {
    MalformedRequest(String),
    UnsupportedMethod(String),
    UnsupportedVersion(String),
    BodyNotAllowed,
    EndOfStream,
    LengthRequired,
    PayloadTooLarge,
}

impl std::fmt::Display for HttpParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            HttpParseError::MalformedRequest(ref s) => {
                write!(f, "Request was malformed and could not be parsed: '{}'", s)
            }
            HttpParseError::UnsupportedMethod(ref s) => {
                write!(f, "Method not supported: '{}'", s)
            }
            HttpParseError::UnsupportedVersion(ref s) => {
                write!(f, "Version not supported: '{}'", s)
            }
            HttpParseError::BodyNotAllowed => {
                write!(f, "Body not supported for this method")
            }
            HttpParseError::EndOfStream => {
                write!(f, "End of stream")
            }
            HttpParseError::LengthRequired => {
                write!(f, "Content-Length header required")
            }
            HttpParseError::PayloadTooLarge => {
                write!(f, "Payload too large")
            }
        }
    }
}

impl From<std::io::Error> for HttpParseError {
    fn from(e: std::io::Error) -> Self {
        HttpParseError::MalformedRequest(e.to_string())
    }
}

impl From<FromUtf8Error> for HttpParseError {
    fn from(e: FromUtf8Error) -> Self {
        HttpParseError::MalformedRequest(e.to_string())
    }
}

impl From<ParseIntError> for HttpParseError {
    fn from(e: ParseIntError) -> Self {
        HttpParseError::MalformedRequest(e.to_string())
    }
}

impl From<http::Error> for HttpParseError {
    fn from(e: http::Error) -> Self {
        HttpParseError::MalformedRequest(e.to_string())
    }
}

impl From<HttpParseError> for StatusCode {
    fn from(e: HttpParseError) -> Self {
        match e {
            HttpParseError::MalformedRequest(_) => StatusCode::BAD_REQUEST,
            HttpParseError::UnsupportedMethod(_) => StatusCode::NOT_IMPLEMENTED,
            HttpParseError::UnsupportedVersion(_) => StatusCode::HTTP_VERSION_NOT_SUPPORTED,
            HttpParseError::BodyNotAllowed => StatusCode::BAD_REQUEST,
            HttpParseError::EndOfStream => StatusCode::BAD_REQUEST,
            HttpParseError::LengthRequired => StatusCode::LENGTH_REQUIRED,
            HttpParseError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }
}
