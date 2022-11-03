use std::string::FromUtf8Error;

#[derive(Debug, Clone)]
pub enum HttpParseError {
    MalformedRequest(String),
    UnsupportedMethod(String),
    UnsupportedVersion(String),
    BodyNotAllowed,
    EndOfStream,
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

impl From<http::Error> for HttpParseError {
    fn from(e: http::Error) -> Self {
        HttpParseError::MalformedRequest(e.to_string())
    }
}
