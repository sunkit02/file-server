use std::fs;

use askama::Template;
use actix_web::{get, http::header::ContentType, HttpResponse, Responder, web::{Data, Path, Query}};
use log::{info, debug};

use crate::{file_manager::templates::{HomePageTemplate, ProgramListTemplate}, configs::ServerConfigs, file_server::{handlers::{DirectoryStructureQuery, get_directory_structure_recursive, get_directory_structure}, models::Directory}};

const CSS_FILE: &'static [u8] = include_bytes!("../../public/css/main.css");

#[get("/")]
pub async fn home_page() -> impl Responder {
    info!("Getting home page");
    let css_content = String::from_utf8(Vec::from(CSS_FILE))
        .unwrap_or("".to_string());
    let template = HomePageTemplate {
        css_content,
    }
    .render()
    .unwrap();

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

#[get("/manager/api/v1/directory-structure/{path:.*}")]
pub async fn directory_structure_template(
    configs: Data<ServerConfigs>,
    path: Path<String>,
    query: Query<DirectoryStructureQuery>
) -> impl Responder {
    info!("Getting directory structure for path: {}", path);
    debug!("Query: {:?}", query);

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
            let template = ProgramListTemplate {
                base_dir
            }
            .render()
            .unwrap();
            return HttpResponse::Ok()
                .insert_header(ContentType::html())
                .body(template);
        },
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
    }
}
