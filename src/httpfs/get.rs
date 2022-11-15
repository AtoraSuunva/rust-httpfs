use std::{collections::HashMap, path::Path};

use http::{header, Response};

use crate::filesystem::{get_directory, get_file, is_directory, DirEntry};

use super::{
    message::{ByteRequest, ByteResponse},
    parse::parse_query,
    server::UnrecoverableError,
};

pub async fn handle_get(
    request: &ByteRequest,
    path: impl AsRef<Path>,
) -> Result<ByteResponse, UnrecoverableError> {
    if is_directory(&path).await {
        serve_directory(request, path).await
    } else {
        serve_file(request, path).await
    }
}

async fn serve_directory(
    request: &ByteRequest,
    path: impl AsRef<Path>,
) -> Result<ByteResponse, UnrecoverableError> {
    let mut entries = match get_directory(&path).await {
        Some(entries) => entries,
        None => return create_404(path.as_ref().to_string_lossy().as_ref()),
    };

    let accept = request.headers().get(header::ACCEPT);
    let mut use_html = false;

    if let Some(accept) = accept {
        let accept = accept.to_str().unwrap_or_default();

        // We should really do actual parsing but this is good enough
        if accept.contains("text/html") {
            use_html = true;
        }
    }

    entries.sort_unstable();

    let body: Vec<u8> = if use_html {
        format_directory_html(&entries)
    } else {
        format_directory_plaintext(&entries)
    };

    let content_type = if use_html { "text/html" } else { "text/plain" };

    let response: ByteResponse = Response::builder()
        .status(200)
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONNECTION, "close")
        .body(Some(body))?;

    Ok(response)
}

fn format_directory_html(entries: &[DirEntry]) -> Vec<u8> {
    let entries = entries
        .iter()
        .map(|e| e.html_format())
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "<!DOCTYPE html>\n<html>\n<head><meta charset=\"UTF-8\"></head><body>\n<ul>\n{}\n</ul>\n</body></html>",
        entries
    )
    .into()
}

fn format_directory_plaintext(entries: &[DirEntry]) -> Vec<u8> {
    entries
        .iter()
        .map(|e| e.plaintext_format())
        .collect::<Vec<String>>()
        .join("\n")
        .into()
}

async fn serve_file(
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
        None => create_404(request.uri().path())?,
    };

    Ok(response)
}

fn create_404(path: &str) -> Result<ByteResponse, UnrecoverableError> {
    let body: Vec<u8> = format!("404: '{}' not found!", path).into();

    Ok(Response::builder()
        .status(404)
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::CONTENT_TYPE, "text/plain")
        .header(header::CONNECTION, "close")
        .body(Some(body))?)
}
