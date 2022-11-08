use std::path::Path;

use mime_guess::Mime;

pub struct ServerFile {
    pub content: Vec<u8>,
    pub mime: Mime,
}

pub async fn get_file(path: impl AsRef<Path>) -> Option<ServerFile> {
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

/// Resolve `.` and `..` in a path
/// ```
/// assert_eq!(flatten_path("/./test"), "/test");
/// assert_eq!(flatten_path("/../test"), "/test");
/// assert_eq!(flatten_path("/foo/./test"), "/foo/test");
/// assert_eq!(flatten_path("/foo/../test"), "/test");
/// assert_eq!(flatten_path("/foo/./../test"), "/test");
/// ```
pub fn flatten_path(path: &str) -> String {
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
