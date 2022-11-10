use std::path::Path;

use super::{
    get::handle_get,
    message::{ByteRequest, ByteResponse},
    server::UnrecoverableError,
};

pub async fn handle_head(
    request: &ByteRequest,
    path: impl AsRef<Path>,
) -> Result<ByteResponse, UnrecoverableError> {
    // HEAD is GET but without body
    // could be optimized by not pulling the body in the first place
    // but this is easier and we dont really need optimization
    let mut response = handle_get(request, path).await?;
    response.body_mut().take();

    Ok(response)
}
