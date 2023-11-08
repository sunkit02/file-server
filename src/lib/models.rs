use std::path::PathBuf;

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
}
