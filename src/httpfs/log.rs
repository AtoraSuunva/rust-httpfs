use std::str;

use http::{header, Method, StatusCode};
use owo_colors::{OwoColorize, Style};

use crate::{
    colorize::MColorize,
    httpfs::message::{
        ByteRequest, ByteResponse, RequestMessage, RequestStyles, ResponseMessage, ResponseStyles,
    },
    httpfs::server::UnrecoverableError,
};

pub fn log_request_response_short(request: &ByteRequest, response: &ByteResponse) {
    println!(
        "{} {} → {} {}",
        request
            .method()
            .out_color(|t| t.style(method_style(request.method()))),
        request.uri().path().out_color(|t| t.cyan()),
        response
            .status()
            .out_color(|t| t.style(status_style(&response.status()))),
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap_or(&"<unknown>".parse().unwrap())
            .to_str()
            .unwrap_or("<unknown>")
            .out_color(|t| t.bright_black())
    );
}

fn method_style(method: &Method) -> Style {
    match *method {
        Method::GET => Style::new().green(),
        Method::POST => Style::new().yellow(),
        Method::PUT => Style::new().blue(),
        Method::PATCH => Style::new().cyan(),
        Method::DELETE => Style::new().red(),
        Method::HEAD => Style::new().bright_black(),
        _ => Style::new(),
    }
}

fn status_style(status: &StatusCode) -> Style {
    match status.as_u16() {
        200..=299 => Style::new().green(),
        300..=399 => Style::new().yellow(),
        400..=499 => Style::new().red(),
        500..=599 => Style::new().bright_red(),
        _ => Style::new(),
    }
}

pub fn log_request(request: &ByteRequest) -> Result<(), UnrecoverableError> {
    let http_message = RequestMessage::from(request);
    let request_styles = RequestStyles::colorized();
    let (req_message, req_body) = http_message.to_parts(&request_styles)?;

    let display_body = if !req_body.is_empty() {
        match str::from_utf8(req_body.as_slice()) {
            Ok(req_body) => format!("{}\n\n", req_body),
            Err(_) => String::from("[Invalid UTF-8]"),
        }
    } else {
        String::new()
    };

    println!("{} {}", req_message, display_body);

    Ok(())
}

pub fn log_response(message: &ResponseMessage) -> Result<(), UnrecoverableError> {
    let colored_styles = ResponseStyles::colorized();
    let (colored_message, colored_body) = message.to_parts(&colored_styles)?;
    let display_body = if !colored_body.is_empty() {
        match str::from_utf8(colored_body.as_slice()) {
            Ok(colored_body) => format!("{}\n\n", colored_body),
            Err(_) => String::from("[Invalid UTF-8]"),
        }
    } else {
        String::new()
    };

    println!("{}{}", colored_message, display_body);

    Ok(())
}
