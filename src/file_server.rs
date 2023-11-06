use actix_files::NamedFile;
use actix_web::{
    self, get,
    http::header,
    web::{Data, Query},
    App, HttpResponse, HttpServer, Responder,
};
use env_logger;
use log::{debug, info};
use mime_guess;
use serde::Deserialize;

use std::env;
use std::io::prelude::*;

use crate::configs::ServerConfigs;

#[derive(Deserialize)]
struct FileRequest {
    path: String,
    force_display: Option<bool>,
}

#[get("/")]
async fn serve_static_file(
    configs: Data<ServerConfigs>,
    file_request: Query<FileRequest>,
) -> impl Responder {
    // TODO: Add request ID for debugging purposes
    info!("Getting file with path: {}", file_request.path);
    debug!("Forced_display: {:?}", file_request.force_display);

    let mut file_path = configs.base_dir.clone();
    file_path.push(&file_request.path);

    let file_result = NamedFile::open(&file_path);

    match file_result {
        Ok(file) => {
            let file = file.file();
            let bytes = file.bytes().map(|byte| byte.unwrap()).collect::<Vec<_>>();

            let mut response_builder = HttpResponse::Ok();

            match file_request.force_display {
                Some(force_display) if force_display => {
                    response_builder.insert_header(header::ContentType::plaintext());
                }
                _ => {
                    // get file mimetype from file name
                    let mime_type = match mime_guess::from_path(file_path).first() {
                        Some(mime) => dbg!(mime).to_string(),
                        None => "text/plain".to_string(),
                    };
                    dbg!(&mime_type);
                    response_builder.insert_header(("Content-Type", mime_type.as_str()));
                }
            }

            response_builder.body(bytes)
        }
        Err(_) => {
            let message = format!("Failed to get file with path: {}", file_request.path);
            info!("{}", message);
            HttpResponse::NotFound().body(message)
        }
    }
}

pub async fn start(configs: ServerConfigs) -> std::io::Result<()> {
    env::set_var("RUST_LOG", configs.log_level.to_string());
    env_logger::init();

    info!("Starting server with configs: {:?}", configs);
    info!(
        "Server will be listening at {}:{}",
        configs.host, configs.port
    );

    let shared_configs = configs.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(shared_configs.clone()))
            .service(serve_static_file)
    })
    .workers(2)
    .bind((configs.host, configs.port))?
    .run()
    .await
}
