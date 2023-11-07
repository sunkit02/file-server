use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Directory {
    pub name: String,
    pub entries: Vec<DirectoryEntry>,
    pub path: PathBuf,
}

#[derive(Debug, Serialize)]
pub enum DirectoryEntry {
    Directory(Directory),
    File {
        name: String,
        path: PathBuf,
    }
}
