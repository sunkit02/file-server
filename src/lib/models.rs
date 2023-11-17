use std::{cmp::Ordering, path::PathBuf};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Directory {
    pub name: String,
    // TODO: Make optional to represent unvisited state (Or use enum?)
    pub entries: Vec<DirectoryEntry>,
    pub path: PathBuf,
}

#[derive(Debug, Serialize)]
pub enum DirectoryEntry {
    Directory(Directory),
    File { name: String, path: PathBuf },
}

impl DirectoryEntry {
    pub fn is_directory(&self) -> bool {
        match self {
            Self::Directory(_) => true,
            Self::File { name: _, path: _ } => false,
        }
    }
}

impl Directory {
    // TODO: Optimize the sanitization process to avoid copying
    pub fn sanitize_path(&mut self, base_path: &str) {
        let mut base_path = base_path.to_owned();
        base_path.push('/');
        Self::remove_base_path(self, &base_path);
    }

    fn remove_base_path(root: &mut Directory, base_path: &str) {
        root.path = PathBuf::from(root.path.to_string_lossy().replacen(base_path, "", 1));
        root.entries.iter_mut().for_each(|entry| match entry {
            DirectoryEntry::Directory(dir) => Self::remove_base_path(dir, base_path),
            DirectoryEntry::File { name: _, path } => {
                *path = PathBuf::from(path.to_string_lossy().replacen(base_path, "", 1))
            }
        })
    }

    pub fn sort_entries(&mut self) {
        use DirectoryEntry::*;
        self.entries.sort_by(|a, b| match (a, b) {
            (Directory(a), Directory(b)) => a.name.cmp(&b.name),
            (
                File {
                    name: a_name,
                    path: _,
                },
                File {
                    name: b_name,
                    path: _,
                },
            ) => a_name.cmp(b_name),
            _ => {
                if a.is_directory() {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
        })
    }
}

#[derive(Debug, Serialize)]
pub enum MediaType {
    TEXT,
    IMAGE,
    AUDIO,
    VIDEO,
    OTHER,
}

impl From<&str> for MediaType {
    fn from(mime_type: &str) -> Self {
        let (mime_type, _subtype) = mime_type.split_once("/").unwrap();
        match mime_type {
            "text"  => MediaType::TEXT,
            "image" => MediaType::IMAGE,
            "audio" => MediaType::AUDIO,
            "video" => MediaType::VIDEO,
            _       => MediaType::OTHER,
        }
    }
}
