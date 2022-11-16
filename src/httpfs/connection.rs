use std::path::{Path, PathBuf};

use http::{header, Method, Response, StatusCode};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use urlencoding::decode;

use crate::{
    cli::VERY_VERBOSE,
    filesystem::flatten_path,
    httpfs::get::handle_get,
    httpfs::head::handle_head,
    httpfs::log::{log_request, log_request_response_short, log_response},
    httpfs::message::{ByteResponse, ResponseMessage, ResponseStyles},
    httpfs::parse::parse_request,
    httpfs::post::handle_post,
    httpfs::server::UnrecoverableError,
};

pub async fn handle_connection(mut stream: TcpStream, directory: &str, verbosity: u8) {
    let mut response: Option<ByteResponse> = None;

    if let Err(e) = handle_request(&mut stream, directory, verbosity).await {
        eprintln!("Error: {}", e);
        let body: Vec<u8> = format!("Error: {}", e).into_bytes();
        response = Some(
            Response::builder()
                .status(500)
                .header(header::CONTENT_LENGTH, body.len())
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONNECTION, "close")
                .body(Some(body))
                .unwrap(),
        );
    };

    // std::error::Error isn't Send so we can't just nest it in the above if
    // and this is the best i could come up with right now so i'm sorry
    if let Some(res) = response {
        if let Err(e2) = write_response(&ResponseMessage::from(&res), &mut stream).await {
            eprintln!("Error writing Error Response: {}", e2);
        };
    }
}

async fn handle_request(
    stream: &mut TcpStream,
    directory: &str,
    verbosity: u8,
) -> Result<(), UnrecoverableError> {
    let request = match parse_request(stream).await {
        Ok(request) => request,
        Err(e) => {
            let body = format!("Request parse error: {}", e);
            eprintln!("{}", body);
            let response: ByteResponse = Response::builder()
                .status(StatusCode::from(e))
                .header(header::CONTENT_LENGTH, body.len())
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONNECTION, "close")
                .body(Some(body.into_bytes()))
                .unwrap();

            let response = ResponseMessage::from(&response);
            write_response(&response, stream).await?;
            return Ok(());
        }
    };

    if verbosity >= VERY_VERBOSE {
        log_request(&request)?;
    }

    // Decode the path so encoded entities like "%20" are turned into " "
    // Filesystem paths dont use entities
    let client_path = PathBuf::from(decode(request.uri().path())?.to_string());
    // Flatten the client's request so `/../Cargo.toml` becomes `/Cargo.toml`
    // This prevents escaping the data directory using `..`
    let client_path = flatten_path(client_path);
    // Join the path to the data directory
    let path = Path::new(directory).join(Path::new(&client_path));

    let response = match *request.method() {
        Method::GET => handle_get(&request, path).await?,
        Method::HEAD => handle_head(&request, path).await?,
        Method::POST => handle_post(&request, path).await?,
        _ => handle_unknown(),
    };

    log_request_response_short(&request, &response);

    let http_message = ResponseMessage::from(&response);

    if verbosity >= VERY_VERBOSE {
        log_response(&http_message)?;
    }

    write_response(&http_message, stream).await?;
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
