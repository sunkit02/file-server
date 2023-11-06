use actix_web::{self, HttpServer, App, Responder, web::{Data, Path}, get};
use actix_files::NamedFile;
use env_logger;
use log::{info, debug, error};

use std::{env, sync::Arc};
use std::path;

#[derive(Debug, Clone)]
struct ServerConfigs {
    base_path: String,
    host: String,
    port: u16, 
}

impl ServerConfigs {
    fn builder() -> ServerConfigsBuilder {
        ServerConfigsBuilder { base_path: None, host: None, port: None }
    }
}

impl Default for ServerConfigs {
    fn default() -> Self {
        Self {
            base_path: "./".to_string(),
            host: "localhost".to_string(),
            port: 8080,
        }
    }
}

#[derive(Debug, Clone)]
struct ServerConfigsBuilder {
    base_path: Option<String>,
    host: Option<String>,
    port: Option<u16>, 
}

impl ServerConfigsBuilder {
    fn base_path(&mut self, path: &str) -> &Self {
        self.base_path = Some(path.to_string());
        self
    }
    fn host(&mut self, host: &str) -> &Self {
        self.host = Some(host.to_string());
        self
    }
    fn port(&mut self, port: u16) -> &Self { 
        self.port = Some(port);
        self
    }
    fn build(mut self) -> ServerConfigs {
        let mut config = ServerConfigs::default();

        if let Some(base_path) = self.base_path.take() {
            config.base_path = base_path;
        }
        if let Some(host) = self.host.take() {
            config.host = host;
        }
        if let Some(port) = self.port.take() {
            config.port = port;
        }

        config
    }
}

#[get("/{path}")]
async fn serve_static_file(configs: Data<ServerConfigs>, path: Path<String>) -> impl Responder {
    info!("Getting file with path: {}", path);

    let mut file_path = configs.base_path.to_string();
    file_path.push_str(&path);
    let file_result = NamedFile::open(file_path);

    if file_result.is_err() {
        info!("Failed to get file with path: {}", path);
    }

    file_result
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let args = env::args().collect::<Vec<_>>();

    let mut configs = ServerConfigs::builder();

    if let Some(base_path) = args.get(1) {
        let path_is_valid = path::Path::new(base_path).exists();
        if path_is_valid {
            configs.base_path(base_path);
        } else {
            error!("Base path '{}' given is invalid. Aborting...", base_path);
            return Ok(());
        }
    }

    if let Some(host) = args.get(2) {
        configs.host(host);
    }

    if let Some(port) = args.get(3) {
        if let Ok(port) = port.parse() {
            configs.port(port);
        }
    }

    let configs = configs.build();
    let shared_configs =  configs.clone();

    info!("Starting server with configs: {:?}", configs);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(shared_configs.clone()))
            .service(serve_static_file)
    })
    .bind((configs.host, configs.port))?
    .run()
    .await
}
