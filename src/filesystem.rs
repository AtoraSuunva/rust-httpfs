use std::path::{Component, Path, PathBuf};

use mime_guess::Mime;
use tokio::fs;

pub struct ServerFile {
    pub content: Vec<u8>,
    pub mime: Mime,
}

pub async fn get_file(path: impl AsRef<Path>) -> Option<ServerFile> {
    let file = match fs::read(&path).await {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    Some(ServerFile {
        content: file,
        mime,
    })
}

#[derive(Eq, PartialEq)]
pub struct DirEntry {
    pub name: String,
    pub is_directory: bool,
    pub mime: Mime,
}

impl PartialOrd for DirEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DirEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let dir_ordering = self.is_directory.cmp(&other.is_directory).reverse();

        if dir_ordering == std::cmp::Ordering::Equal {
            self.name.cmp(&other.name)
        } else {
            dir_ordering
        }
    }
}

pub async fn get_directory(path: impl AsRef<Path>) -> Option<Vec<DirEntry>> {
    let mut entries = match fs::read_dir(&path).await {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    let mut res: Vec<DirEntry> = vec![];

    loop {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            // simply not our problem
            Err(_) => continue,
        };

        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        let mime = mime_guess::from_path(entry.file_name()).first_or_octet_stream();
        let is_directory = entry.file_type().await.unwrap().is_dir();

        res.push(DirEntry {
            name,
            mime,
            is_directory,
        });
    }

    Some(res)
}

pub async fn is_directory(path: impl AsRef<Path>) -> bool {
    let metadata = fs::metadata(&path).await;
    metadata.map(|m| m.is_dir()).unwrap_or(false)
}

/// Resolve `.` and `..` in a path
/// ```
/// assert_eq!(flatten_path(PathBuf::from("/./test")), PathBuf::from("/test")));
/// assert_eq!(flatten_path(PathBuf::from("/../test")), PathBuf::from("/test"));
/// assert_eq!(flatten_path(PathBuf::from("/foo/./test")), PathBuf::from("/foo/test"));
/// assert_eq!(flatten_path(PathBuf::from("/foo/../test")), PathBuf::from("/test"));
/// assert_eq!(flatten_path(PathBuf::from("/foo/./../test")), PathBuf::from("/test"));
/// ```
pub fn flatten_path(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .components()
        // we can just ignore `.` since it doesn't change the path
        // and we ignore the root dir because we don't need it
        .filter(|x| x != &Component::RootDir && x != &Component::CurDir)
        .fold(PathBuf::new(), |mut acc, x| {
            if x == Component::ParentDir {
                acc.pop();
            } else {
                acc.push(x);
            }
            acc
        })
}
