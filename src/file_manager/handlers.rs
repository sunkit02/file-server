use std::{fs, path::PathBuf};

use actix_web::{get, http::header::ContentType, HttpResponse, Responder, web::{Data, Query, Path}};
use log::info;
use serde::Deserialize;

use crate::configs::ServerConfigs;

use super::models::{DirectoryEntry, Directory};

#[get("/")]
pub async fn home_page() -> impl Responder {
    info!("Getting home page");
    HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body("<h1>Home Page</h1>")
}

#[get("/favicon.ico")]
pub async fn favicon() -> impl Responder {
    let favicon_bytes = include_bytes!("../../public/folder.svg").as_slice();
    HttpResponse::Ok()
        .insert_header(("Content-Type", "image/svg+xml"))
        .body(favicon_bytes)
}

#[derive(Debug, Deserialize)]
pub struct DirectoryStructureQuery {
    recursive: Option<bool>,
}

#[get("/api/v1/directory-structure/{path:.*}")]
pub async fn dir_structure(
    configs: Data<ServerConfigs>,
    path: Path<String>,
    query: Query<DirectoryStructureQuery>
) -> impl Responder {
    info!("Getting directory structure for path: {}", path);

    let mut root_dir_path = configs.base_dir.clone();
    root_dir_path.push(path.as_str());

    let metadata = match fs::metadata(&root_dir_path) {
        Ok(metadata) => metadata,
        Err(_) => {
        return HttpResponse::BadRequest()
            .body(format!("Failed to get metadata for: {:?}", root_dir_path))
        }
    };

    if !metadata.is_dir() {
        return HttpResponse::BadRequest()
            .body(format!("{:?} is not a directory", root_dir_path));
    };

    let name = match root_dir_path.file_name() {
        Some(file_name) => file_name.to_str().unwrap_or("Unknown Filename").to_owned(),
        None => "Unknown Filename".to_owned(),
    };

    let mut base_dir = Directory {
        name,
        path: root_dir_path,
        entries: Vec::new(),
    };

    let get_dir_structure_result = match query.recursive {
        Some(recursive) if recursive => {
            get_directory_structure_recursive(&mut base_dir)
        },
        _ => get_directory_structure(&mut base_dir),
    };
    
    match get_dir_structure_result {
        Ok(_) => {
            base_dir.sanitize_path(&configs.base_dir.to_string_lossy());
            return HttpResponse::Ok()
                .insert_header(ContentType::json())
                .body(serde_json::to_string(&base_dir).unwrap());
        },
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
    }
}

fn get_directory_structure(root_directory: &mut Directory) -> std::io::Result<()> {
    let root_entries = fs::read_dir(&root_directory.path)?.flatten();

    for root_entry in root_entries {
        let root_dir_entry: DirectoryEntry;

        let name = root_entry.file_name().to_str().unwrap_or("Unknown Filename").to_owned();
        let path = PathBuf::from(root_entry.path().to_str().unwrap_or("Unknown Path"));

        let is_directory = match root_entry.metadata() {
            Ok(metadata) => metadata.is_dir(),
            _ => false,
        };

        if is_directory {
            root_dir_entry = DirectoryEntry::Directory(Directory {
                name,
                entries: Vec::new(),
                path,
            });
        } else {
            root_dir_entry = DirectoryEntry::File { name, path };
        }

        root_directory.entries.push(root_dir_entry);
    }

    Ok(())
}

fn get_directory_structure_recursive(root_directory: &mut Directory) -> std::io::Result<()> {
    let root_entries = fs::read_dir(&root_directory.path)?.flatten();

    for root_entry in root_entries {
        let root_dir_entry: DirectoryEntry;

        let name = root_entry.file_name().to_str().unwrap_or("Unknown Filename").to_owned();
        let path = PathBuf::from(root_entry.path().to_str().unwrap_or("Unknown Path"));

        let is_directory = match root_entry.metadata() {
            Ok(metadata) => metadata.is_dir(),
            _ => false,
        };

        if is_directory {
            let mut directory = Directory {
                name,
                entries: Vec::new(),
                path,
            };

            // Recursively check for entries if is directory
            let _ = get_directory_structure_recursive(&mut directory);
            root_dir_entry = DirectoryEntry::Directory(directory);
        } else {
            root_dir_entry = DirectoryEntry::File { name, path };
        }

        root_directory.entries.push(root_dir_entry);
    }

    Ok(())
}
