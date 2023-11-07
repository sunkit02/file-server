use actix_files::NamedFile;
use actix_web::{
    get,
    http::header::ContentType,
    web::{Data, Query, Path},
    HttpResponse, Responder,
};
use log::{debug, info};
use mime_guess;
use serde::Deserialize;

use std::io::prelude::*;

use crate::configs::ServerConfigs;

#[derive(Deserialize)]
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
    info!("Getting file with path: {}", path);
    debug!("Forced_display: {:?}", query.force_display);

    let mut file_path = configs.base_dir.clone();
    file_path.push(path.as_str());

    let file_bytes = match NamedFile::open(&file_path) {
        Ok(file) => {
            let file = file.file();
            file.bytes().map(|byte| byte.unwrap()).collect::<Vec<_>>()
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
        }
        _ => {
            // get file mimetype from file name
            let mime_type = match mime_guess::from_path(file_path).first() {
                Some(mime) => dbg!(mime).to_string(),
                None => "text/plain".to_string(),
            };
            response_builder.insert_header(("Content-Type", mime_type.as_str()));
        }
    }

    response_builder.body(file_bytes)
}
