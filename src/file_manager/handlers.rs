use std::{fs, path::PathBuf};

use actix_web::{
    get,
    http::header::ContentType,
    web::{Data, Path, Query},
    HttpResponse, Responder,
};
use askama::Template;
use file_server_core::*;

use serde::Deserialize;

use crate::file_manager::templates::{HomePageTemplate, ProgramListTemplate};
use crate::{
    configs::ServerConfigs,
    file_manager::templates::{DirectoryTemplate, FileContentTemplate},
};

const CSS_FILE: &[u8] = include_bytes!("../../public/css/main.css");

#[get("/")]
pub async fn home_page() -> impl Responder {
    let css_content = String::from_utf8(Vec::from(CSS_FILE)).unwrap_or("".to_string());
    let template = HomePageTemplate { css_content }.render().unwrap();

    HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body(template)
}

#[get("/favicon.ico")]
pub async fn favicon() -> impl Responder {
    let favicon_bytes = include_bytes!("../../public/folder.svg").as_slice();
    HttpResponse::Ok()
        .insert_header(("Content-Type", "image/svg+xml"))
        .body(favicon_bytes)
}

#[derive(Debug, Deserialize)]
pub struct FileManagerDirectoryStructureQuery {
    pub recursive: Option<bool>,
    pub expanded: Option<bool>,
}

#[get("/manager/api/v1/directory-structure/{path:.*}")]
pub async fn directory_structure_template(
    configs: Data<ServerConfigs>,
    path: Path<String>,
    query: Query<FileManagerDirectoryStructureQuery>,
) -> impl Responder {
    let mut root_dir_path = configs.base_dir.clone();

    // Remove prefix '/' for queries not pointing to base_dir
    // TODO: Solve this in a cleaner way (at the type level)
    let mut path = path.into_inner();
    if !path.is_empty() && path.starts_with('/') {
        // Remove the '/'
        path.remove(0);
    }

    root_dir_path.push(path);

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

    // Return early if expanded
    if let Some(expanded) = query.expanded {
        if expanded {
            base_dir.sanitize_path(&configs.base_dir.to_string_lossy());

            let template = ProgramListTemplate {
                base_dir: DirectoryTemplate::from(&base_dir),
                expanded: false,
            }
            .render()
            .unwrap();

            return HttpResponse::Ok()
                .insert_header(ContentType::html())
                .body(template);
        }
    }

    let get_dir_structure_result = match query.recursive {
        Some(recursive) if recursive => get_directory_structure_recursive(&mut base_dir),
        _ => get_directory_structure(&mut base_dir),
    };

    base_dir.sort_entries();

    match get_dir_structure_result {
        Ok(_) => {
            base_dir.sanitize_path(&configs.base_dir.to_string_lossy());

            let template = ProgramListTemplate {
                base_dir: DirectoryTemplate::from(&base_dir),
                expanded: true,
            }
            .render()
            .unwrap();

            return HttpResponse::Ok()
                .insert_header(ContentType::plaintext())
                .body(template);
        }
        Err(err) => HttpResponse::BadRequest().body(err.to_string()),
    }
}

#[get("/manager/api/v1/file-content/{path:.*}")]
pub async fn file_content(path: Path<PathBuf>) -> impl Responder {
    let path = path.into_inner();
    let name = path.file_name().unwrap().to_str().unwrap();
    let mime_type = mime_guess::from_path(name).first_raw().unwrap_or("text/plain");
    let media_type = MediaType::from(mime_type);

    let template = FileContentTemplate {
        name,
        path: path.to_str().unwrap_or(name),
        media_type,
    }
    .render()
    .unwrap();

    HttpResponse::Ok()
        .insert_header(ContentType::plaintext())
        .body(template)
}
