use std::path::Path;

use http::{header, Response};
use tokio::fs;

use super::{
    message::{ByteRequest, ByteResponse},
    server::UnrecoverableError,
};

pub async fn handle_post(
    request: &ByteRequest,
    path: impl AsRef<Path>,
) -> Result<ByteResponse, UnrecoverableError> {
    // Create any required parent directories
    // Path should already be flattened to our data directory, so this shouldn't
    // be able to escape the data directory and create undesired paths
    if let Some(p) = path.as_ref().parent() {
        fs::create_dir_all(p).await?
    };

    // If you somehow provide no body, we just write nothing
    // There might be a better way to do this...
    // But this method does not allocate anything in the vec, so it should be ok?
    // Maybe having a required body for POST would be better, but then we'd have more
    // than just a ByteRequest, which would be annoying to work with
    let empty: Vec<u8> = Vec::new();
    let content = request.body().as_ref().unwrap_or_else(|| empty.as_ref());
    fs::write(path, content).await?;

    let body: Vec<u8> = format!("201: '{}' created!", request.uri().path()).into();
    let response: ByteResponse = Response::builder()
        .status(201)
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::CONTENT_TYPE, "text/plain")
        .header(header::CONNECTION, "close")
        .body(Some(body))?;

    Ok(response)
}
