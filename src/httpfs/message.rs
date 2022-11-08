use std::fmt::Write;

use http::{HeaderMap, Request, Response};
use owo_colors::{OwoColorize, Style};

pub type ByteRequest = Request<Option<Vec<u8>>>;
pub type ByteResponse = Response<Option<Vec<u8>>>;

#[derive(Debug, Default)]
pub struct RequestStyles {
    method: Style,
    abs_path: Style,
    version: Style,
    header_name: Style,
    header_value: Style,
}

impl RequestStyles {
    pub fn colorized() -> Self {
        Self {
            method: Style::new().green(),
            abs_path: Style::new().blue(),
            version: Style::new().bright_black(),
            header_name: Style::new().cyan(),
            header_value: Style::new().purple(),
        }
    }
}

pub struct RequestMessage {
    method: String,
    abs_path: String,
    version: String,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
}

impl RequestMessage {
    pub fn to_parts(&self, styles: &RequestStyles) -> Result<(String, Vec<u8>), std::fmt::Error> {
        let mut message = String::new();

        write!(
            message,
            "{} {} {}\r\n",
            self.method.style(styles.method),
            self.abs_path.style(styles.abs_path),
            self.version.style(styles.version),
        )?;

        for (name, value) in &self.headers {
            write!(
                message,
                "{}: {}\r\n",
                name.style(styles.header_name),
                value.to_str().unwrap().style(styles.header_value),
            )?;
        }

        message.push_str("\r\n");

        Ok((message, self.body.clone().unwrap_or_default()))
    }
}

impl From<&ByteRequest> for RequestMessage {
    fn from(req: &ByteRequest) -> Self {
        let method = req.method().to_string();
        let abs_path = req.uri().path_and_query().unwrap().to_string();
        let version = format!("{:?}", req.version());
        let headers = req.headers().to_owned();
        let body = req.body().to_owned();

        Self {
            method,
            abs_path,
            version,
            headers,
            body,
        }
    }
}

pub struct ResponseMessage {
    version: String,
    status: String,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
}

#[derive(Debug, Default)]
pub struct ResponseStyles {
    version: Style,
    status: Style,
    header_name: Style,
    header_value: Style,
}

impl ResponseStyles {
    pub fn colorized() -> Self {
        Self {
            version: Style::new().bright_black(),
            status: Style::new().green(),
            header_name: Style::new().cyan(),
            header_value: Style::new().purple(),
        }
    }
}

impl ResponseMessage {
    pub fn to_parts(&self, styles: &ResponseStyles) -> Result<(String, Vec<u8>), std::fmt::Error> {
        let mut message = String::new();

        write!(
            message,
            "{} {}\r\n",
            self.version.style(styles.version),
            self.status.style(styles.status),
        )?;

        for (name, value) in &self.headers {
            write!(
                message,
                "{}: {}\r\n",
                name.style(styles.header_name),
                value.to_str().unwrap().style(styles.header_value),
            )?;
        }

        message.push_str("\r\n");

        Ok((message, self.body.clone().unwrap_or_default()))
    }
}

impl From<&ByteResponse> for ResponseMessage {
    fn from(res: &ByteResponse) -> Self {
        let version = format!("{:?}", res.version());
        let status = format!("{:?}", res.status());
        let headers = res.headers().to_owned();
        let body = res.body().to_owned();

        Self {
            version,
            status,
            headers,
            body,
        }
    }
}
