use std::{collections::HashMap, path::Path};

use http::{header, Response};

use crate::filesystem::get_file;

use super::{
    message::{ByteRequest, ByteResponse},
    parse::parse_query,
    server::UnrecoverableError,
};

pub async fn handle_get(
    request: &ByteRequest,
    path: impl AsRef<Path>,
) -> Result<ByteResponse, UnrecoverableError> {
    let file = get_file(&path).await;

    let query = request.uri().query().map_or_else(HashMap::new, parse_query);

    // maybe this should be a boolean like ?download=1 or ?download=0
    // but who cares, there isn't some kind of spec we need to follow here
    // and i don't want to have ["false", "0", "no", "FALSE", etc] as "falsy"
    let disposition = if query.get("download").is_some() {
        "attachment"
    } else {
        "inline"
    };

    let response: ByteResponse = match file {
        Some(file) => Response::builder()
            .status(200)
            .header(header::CONTENT_LENGTH, file.content.len())
            .header(header::CONTENT_TYPE, file.mime.essence_str())
            .header(header::CONTENT_DISPOSITION, disposition)
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
