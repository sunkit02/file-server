use std::{cmp::Ordering, fs::{File, Metadata}, io::Read, path::PathBuf, task::Poll};

use actix_web::web::Bytes;
use futures_util::Stream;
use log::debug;
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
            "text" => MediaType::TEXT,
            "image" => MediaType::IMAGE,
            "audio" => MediaType::AUDIO,
            "video" => MediaType::VIDEO,
            _ => MediaType::OTHER,
        }
    }
}

#[derive(Debug)]
pub struct FileStream {
    pub file: File,
    metadata: Metadata,
    bytes_read: usize,
}

impl FileStream {
    pub fn new(file: File) -> Self {
        let metadata = file.metadata().unwrap();
        Self {
            file,
            metadata,
            bytes_read: 0,
        }
    }
}

impl Stream for FileStream {
    type Item = Result<Bytes, std::io::Error>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let hint = (0, Some(self.metadata.len() as usize - self.bytes_read));

        hint
    }

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut buffer = [0;1024 * 500]; // 500KB
        match self.file.read(&mut buffer) {
            Ok(n) if n == 0 => {
                debug!("Read nothing");
                return Poll::Ready(None)
            }
            Ok(n) => self.bytes_read += n,
            Err(err) => {
                debug!("{}", err);
                return Poll::Ready(Some(Err(err)))
            }
        };

        let buffer = Bytes::from(Vec::from(buffer));

        debug!("Returning {:?} bytes, bytes read: {}", buffer.len(), self.bytes_read);

        Poll::Ready(Some(Ok(buffer)))
    }
}
