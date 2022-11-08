use std::path::PathBuf;

use http::{header, Response};

use crate::{
    filesystem::get_file,
    httpfs::message::{ByteRequest, ByteResponse},
    httpfs::server::UnrecoverableError,
};

pub async fn handle_get(
    request: &ByteRequest,
    path: PathBuf,
) -> Result<ByteResponse, UnrecoverableError> {
    let file = get_file(&path).await;

    let response: ByteResponse = match file {
        Some(file) => Response::builder()
            .status(200)
            .header(header::CONTENT_LENGTH, file.content.len())
            .header(header::CONTENT_TYPE, file.mime.essence_str())
            .header(header::CONNECTION, "close")
            .body(Some(file.content))?,
        None => {
            let body: Vec<u8> = format!("404: '{}' not found!", request.uri().path()).into();
            Response::builder()
                .status(404)
                .header(header::CONTENT_LENGTH, body.len())
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONNECTION, "close")
                .body(Some(body))?
        }
    };

    Ok(response)
}
