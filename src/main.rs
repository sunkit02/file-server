use actix_web::{self, HttpServer, App, Responder, web::{Data, Query}, get, HttpRequest, HttpResponse};
use actix_files::NamedFile;
use env_logger;
use log::{info, error};
use serde::Deserialize;
use clap::{self, command, arg, value_parser};

use std::{env, path::PathBuf};
use std::path;

#[derive(Debug, Clone)]
struct ServerConfigs {
    base_dir: PathBuf,
    host: String,
    port: u16, 
    log_level: log::Level,
}

impl ServerConfigs {
    fn builder() -> ServerConfigsBuilder {
        ServerConfigsBuilder { 
            base_dir: None,
            host: None,
            port: None,
            log_level: None,
        }
    }

    fn from_env_args() -> Self {
        let matches = command!()
            .arg(
                arg!([base_dir] "Optional base directory to serve")
                .required(false)
                .value_parser(value_parser!(PathBuf))
            )
            .arg(
                arg!(-p --port <PORT> "Sets custom port")
                    .required(false)
                    .value_parser(value_parser!(u16))
            )
            .arg(
                arg!(-H --host <HOST> "Sets custom host")
                .required(false)
                    .value_parser(value_parser!(String))
            )
            .arg(
                arg!(-l --loglevel <LOGLEVEL> "Sets log level")
                .required(false)
                    .value_parser(value_parser!(String))
            )
            .get_matches();

        let mut configs_builder = Self::builder();
        
        if let Some(base_dir) = matches.get_one::<PathBuf>("base_dir") {
            if !base_dir.exists() {
                println!("Error: {:?} does not exist.", base_dir);
                std::process::exit(1);
            }

            if !base_dir.is_dir() {
                println!("Error: base_dir is expected to be a directory.");
                println!("{:?} is not a directory.", base_dir);
                std::process::exit(2);
            }

            configs_builder.base_dir(base_dir);
        }

        if let Some(&port) = matches.get_one::<u16>("port") {
            configs_builder.port(port);
        }

        if let Some(host) = matches.get_one::<String>("host") {
            configs_builder.host(host);
        }

        if let Some(log_level) = matches.get_one::<String>("loglevel") {
            configs_builder.log_level(log_level);
        }

        configs_builder.build()
    }
}

impl Default for ServerConfigs {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("./"),
            host: "127.0.0.1".to_string(),
            port: 8080,
            log_level: log::Level::Info,
        }
    }
}

#[derive(Debug, Clone)]
struct ServerConfigsBuilder {
    base_dir: Option<PathBuf>,
    host: Option<String>,
    port: Option<u16>, 
    log_level: Option<log::Level>,
}

impl ServerConfigsBuilder {
    fn base_dir(&mut self, path: &PathBuf) -> &Self {
        self.base_dir = Some(path.to_owned());
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
    fn log_level(&mut self, log_level: &str) -> &Self {
        let log_level = match log_level {
            "error" => log::Level::Error,
            "warn"  => log::Level::Warn,
            "debug" => log::Level::Debug,
            "trace" => log::Level::Trace,
            _       => log::Level::Info,
        };
        self.log_level = Some(log_level);
        self
    }
    fn build(mut self) -> ServerConfigs {
        let mut config = ServerConfigs::default();

        if let Some(base_path) = self.base_dir.take() {
            config.base_dir = base_path;
        }
        if let Some(host) = self.host.take() {
            config.host = host;
        }
        if let Some(port) = self.port.take() {
            config.port = port;
        }
        if let Some(log_level) = self.log_level.take() {
            config.log_level = log_level;
        }

        config
    }
}

#[derive(Deserialize)]
struct FileRequest {
    path: String,
    forced_display: bool,
}

#[get("/")]
async fn serve_static_file(
    configs: Data<ServerConfigs>,
    req: HttpRequest,
    file_request: Query<FileRequest>,
) -> impl Responder {
    info!("Getting file with path: {}, forced_display: {}",
        file_request.path, file_request.forced_display);

    let mut file_path = configs.base_dir.clone();
    file_path.push(&file_request.path);

    let file_result = NamedFile::open(dbg!(file_path));

    match file_result {
        Ok(file) => {
            file.into_response(&req)
        },
        Err(_) => {
            let message = format!("Failed to get file with path: {}", file_request.path);
            info!("{}", message);
            HttpResponse::NotFound()
                .body(message)
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Read configs from cli
    let configs = ServerConfigs::from_env_args();
    let shared_configs =  configs.clone();

    env::set_var("RUST_LOG", configs.log_level.to_string());
    env_logger::init();

    info!("Starting server with configs: {:?}", configs);
    info!("Server will be listening at {}:{}", configs.host, configs.port);

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
