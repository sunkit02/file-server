use std::{fs, path::PathBuf};

pub use models::*;

pub mod models;

pub fn get_directory_structure(root_directory: &mut Directory) -> std::io::Result<()> {
    let root_entries = fs::read_dir(&root_directory.path)?.flatten();

    for root_entry in root_entries {
        let name = root_entry
            .file_name()
            .to_str()
            .unwrap_or("Unknown Filename")
            .to_owned();
        let path = PathBuf::from(root_entry.path().to_str().unwrap_or("Unknown Path"));

        let is_directory = match root_entry.metadata() {
            Ok(metadata) => metadata.is_dir(),
            _ => false,
        };

        let root_dir_entry = if is_directory {
            DirectoryEntry::Directory(Directory {
                name,
                entries: Vec::new(),
                path,
            })
        } else {
            DirectoryEntry::File { name, path }
        };

        root_directory.entries.push(root_dir_entry);
    }

    Ok(())
}

pub fn get_directory_structure_recursive(root_directory: &mut Directory) -> std::io::Result<()> {
    let root_entries = fs::read_dir(&root_directory.path)?.flatten();

    for root_entry in root_entries {
        let name = root_entry
            .file_name()
            .to_str()
            .unwrap_or("Unknown Filename")
            .to_owned();
        let path = PathBuf::from(root_entry.path().to_str().unwrap_or("Unknown Path"));

        let is_directory = match root_entry.metadata() {
            Ok(metadata) => metadata.is_dir(),
            _ => false,
        };

        let root_dir_entry = if is_directory {
            let mut directory = Directory {
                name,
                entries: Vec::new(),
                path,
            };

            // Recursively check for entries if is directory
            let _ = get_directory_structure_recursive(&mut directory);
            DirectoryEntry::Directory(directory)
        } else {
            DirectoryEntry::File { name, path }
        };

        root_directory.entries.push(root_dir_entry);
    }

    Ok(())
}
