use std::{path::Path, str::from_utf8};

use http::{header, Response};
use mime_guess::Mime;
use owo_colors::OwoColorize;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    cli::VERY_VERBOSE,
    colorize::MColorize,
    http_message::{
        ByteRequest, ByteResponse, RequestMessage, RequestStyles, ResponseMessage, ResponseStyles,
    },
    http_parse::parse_request,
    http_server::UnrecoverableError,
};

pub async fn handle_connection(
    mut stream: TcpStream,
    directory: &str,
    verbosity: u8,
) -> Result<(), UnrecoverableError> {
    let request = match parse_request(&mut stream).await {
        Ok(request) => request,
        Err(e) => {
            let body = format!("Request parse error: {}", e);
            eprintln!("{}", body);
            let response: ByteResponse = Response::builder()
                .status(404)
                .header(header::CONTENT_LENGTH, body.len())
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONNECTION, "close")
                .body(Some(body.into_bytes()))
                .unwrap();

            write_response(&response, verbosity, &mut stream).await?;
            return Ok(());
        }
    };

    log_request(&request, verbosity)?;

    // Flatten the client's request so `/../Cargo.toml` becomes `/Cargo.toml`
    // This prevents escaping the data directory using `..`
    let client_path = flatten_path(request.uri().path());
    // Strip the leading `/` from the path so we can join it later (dir.join("/foo") == "/foo", which we don't want)
    let path = Path::new(&client_path).strip_prefix("/")?;
    // Join the path to the data directory
    let path = Path::new(directory).join(path);
    let file = get_file(path).await;

    let response: ByteResponse = match file {
        Some(file) => Response::builder()
            .status(200)
            .header(header::CONTENT_LENGTH, file.content.len())
            .header(header::CONTENT_TYPE, file.mime.essence_str())
            .header(header::CONNECTION, "close")
            .body(Some(file.content))?,
        None => {
            let body: Vec<u8> = format!("404: '{}' not found!", client_path).into();
            Response::builder()
                .status(404)
                .header(header::CONTENT_LENGTH, body.len())
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONNECTION, "close")
                .body(Some(body))?
        }
    };

    write_response(&response, verbosity, &mut stream).await?;

    Ok(())
}

fn log_request(request: &ByteRequest, verbosity: u8) -> Result<(), UnrecoverableError> {
    let http_message = RequestMessage::from(request);
    let request_styles = RequestStyles::colorized();
    let (req_message, req_body) = http_message.to_parts(&request_styles)?;

    if verbosity >= VERY_VERBOSE {
        let display_body = if !req_body.is_empty() {
            match from_utf8(req_body.as_slice()) {
                Ok(req_body) => format!("{}\n\n", req_body),
                Err(_) => String::from("[Invalid UTF-8]"),
            }
        } else {
            String::new()
        };

        println!("{} {}", req_message, display_body);
    } else {
        println!(
            "{} {}",
            request.method().out_color(|t| t.green()),
            request.uri().out_color(|t| t.blue())
        );
    }

    Ok(())
}

struct ServerFile {
    content: Vec<u8>,
    mime: Mime,
}

async fn get_file(path: impl AsRef<Path>) -> Option<ServerFile> {
    let file = match tokio::fs::read(&path).await {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    Some(ServerFile {
        content: file,
        mime,
    })
}

async fn write_response(
    response: &ByteResponse,
    verbosity: u8,
    stream: &mut TcpStream,
) -> Result<(), UnrecoverableError> {
    let http_message = ResponseMessage::from(response);
    let response_styles = ResponseStyles::default();
    let (res_message, res_body) = http_message.to_parts(&response_styles)?;

    if verbosity >= VERY_VERBOSE {
        let colored_styles = ResponseStyles::colorized();
        let (colored_message, colored_body) = http_message.to_parts(&colored_styles)?;
        let display_body = if !colored_body.is_empty() {
            match from_utf8(colored_body.as_slice()) {
                Ok(colored_body) => format!("{}\n\n", colored_body),
                Err(_) => String::from("[Invalid UTF-8]"),
            }
        } else {
            String::new()
        };

        println!("{}{}", colored_message, display_body);
    } else {
        println!("{}", response.status().out_color(|t| t.green()),);
    }

    stream.write_all(res_message.as_bytes()).await?;
    stream.write_all(res_body.as_slice()).await?;

    Ok(())
}

/// Resolve `.` and `..` in a path
/// ```
/// assert_eq!(flatten_path("/./test"), "/test");
/// assert_eq!(flatten_path("/../test"), "/test");
/// assert_eq!(flatten_path("/foo/./test"), "/foo/test");
/// assert_eq!(flatten_path("/foo/../test"), "/test");
/// assert_eq!(flatten_path("/foo/./../test"), "/test");
/// ```
fn flatten_path(path: &str) -> String {
    let path = path
        .split('/')
        .skip(1) // skip leading '/', it gives us an empty string that only gives us pain when we fold
        .filter(|x| x != &".") // we can just ignore `.` since it doesn't change the path
        .fold(vec![], |mut acc, x| {
            if x == ".." {
                acc.pop();
            } else {
                acc.push(x);
            }
            acc
        })
        .join("/");

    // Add leading '/' back, this makes sure we always have it and that
    // `assert_eq!(flatten_path("/.."), "/")` instead of `""`
    format!("/{}", path)
}
