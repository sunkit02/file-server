use actix_files::NamedFile;
use actix_web::{
    get,
    http::header::ContentType,
    web::{Data, Path, Query},
    HttpResponse, Responder,
};
use file_server_core::*;
use log::info;
use mime_guess;
use serde::Deserialize;

use std::fs;
use std::io::Read;

use crate::configs::ServerConfigs;

#[get("/health-check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[derive(Debug, Deserialize)]
struct FileRequest {
    #[serde(rename = "force-display")]
    force_display: Option<bool>,
}

#[get("/api/v1/files/{path:.*}")]
async fn serve_static_file(
    configs: Data<ServerConfigs>,
    path: Path<String>,
    query: Query<FileRequest>,
) -> impl Responder {
    // TODO: Add request ID for debugging purposes
    let mut file_path = configs.base_dir.clone();
    file_path.push(path.as_str());

    let file_bytes = match NamedFile::open_async(&file_path).await {
        Ok(file) => {
            if file.metadata().is_dir() {
                return HttpResponse::BadRequest().body(format!("{:?} is a directory", &file_path));
            }

            file.file()
                .bytes()
                .map(|byte| byte.unwrap())
                .collect::<Vec<_>>()
        }
        Err(_) => {
            let message = format!("Failed to get file with path: {}", path);
            info!("{}", message);

            return HttpResponse::NotFound().body(message);
        }
    };

    let mut response_builder = HttpResponse::Ok();

    // Determine if `text/plain` is used to force browser to display the file contents
    match query.force_display {
        Some(force_display) if force_display => {
            response_builder.insert_header(ContentType::plaintext());
            let file_content = String::from_utf8_lossy(&file_bytes);
            let mut sanitized_file_content = String::with_capacity(file_content.len() * 2);
            file_content.chars().for_each(|c| match c {
                '<' => sanitized_file_content.push_str("&lt;"),
                '>' => sanitized_file_content.push_str("&gt;"),
                '&' => sanitized_file_content.push_str("&amp;"),
                c => sanitized_file_content.push(c),
            });

            response_builder.body(sanitized_file_content)
        }
        _ => {
            // get file mimetype from file name
            let mime_type = match mime_guess::from_path(file_path).first() {
                Some(mime) => mime.to_string(),
                None => "text/plain".to_string(),
            };
            response_builder.insert_header(("Content-Type", mime_type.as_str()));
            response_builder.body(file_bytes)
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct DirectoryStructureQuery {
    pub recursive: Option<bool>,
}

#[get("/api/v1/directory-structure/{path:.*}")]
pub async fn dir_structure(
    configs: Data<ServerConfigs>,
    path: Path<String>,
    query: Query<DirectoryStructureQuery>,
) -> impl Responder {
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
        return HttpResponse::BadRequest().body(format!("{:?} is not a directory", root_dir_path));
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
        Some(recursive) if recursive => get_directory_structure_recursive(&mut base_dir),
        _ => get_directory_structure(&mut base_dir),
    };

    match get_dir_structure_result {
        Ok(_) => {
            base_dir.sanitize_path(&configs.base_dir.to_string_lossy());
            return HttpResponse::Ok()
                .insert_header(ContentType::json())
                .body(serde_json::to_string(&base_dir).unwrap());
        }
        Err(err) => HttpResponse::BadRequest().body(err.to_string()),
    }
}
