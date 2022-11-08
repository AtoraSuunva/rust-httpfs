use std::path::Path;

use http::{header, Method, Response};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    cli::VERY_VERBOSE,
    filesystem::flatten_path,
    httpfs::get::handle_get,
    httpfs::log::{log_request, log_request_response_short, log_response},
    httpfs::message::{ByteResponse, ResponseMessage, ResponseStyles},
    httpfs::parse::parse_request,
    httpfs::server::UnrecoverableError,
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

            let response = ResponseMessage::from(&response);
            write_response(&response, &mut stream).await?;
            return Ok(());
        }
    };

    if verbosity >= VERY_VERBOSE {
        log_request(&request)?;
    }

    // Flatten the client's request so `/../Cargo.toml` becomes `/Cargo.toml`
    // This prevents escaping the data directory using `..`
    let client_path = flatten_path(request.uri().path());
    // Strip the leading `/` from the path so we can join it later (dir.join("/foo") == "/foo", which we don't want)
    let path = Path::new(&client_path).strip_prefix("/")?;
    // Join the path to the data directory
    let path = Path::new(directory).join(path);

    let response = match *request.method() {
        Method::GET => handle_get(&request, path).await?,
        // TODO: support HEAD since it's required
        _ => handle_unknown(),
    };

    log_request_response_short(&request, &response);

    let http_message = ResponseMessage::from(&response);

    if verbosity >= VERY_VERBOSE {
        log_response(&http_message)?;
    }

    write_response(&http_message, &mut stream).await?;
    Ok(())
}

async fn write_response(
    message: &ResponseMessage,
    stream: &mut TcpStream,
) -> Result<(), UnrecoverableError> {
    let response_styles = ResponseStyles::default();
    let (res_message, res_body) = message.to_parts(&response_styles)?;

    stream.write_all(res_message.as_bytes()).await?;
    stream.write_all(res_body.as_slice()).await?;

    Ok(())
}

fn handle_unknown() -> ByteResponse {
    let body: Vec<u8> = "Unknown method!".into();
    Response::builder()
        .status(501)
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::CONTENT_TYPE, "text/plain")
        .header(header::CONNECTION, "close")
        .body(Some(body))
        .unwrap()
}
