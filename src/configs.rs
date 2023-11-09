use clap::{arg, command, value_parser};

use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ServerConfigs {
    pub base_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub log_level: log::Level,
    pub workers: usize,
}

impl Default for ServerConfigs {
    fn default() -> Self {
        Self {
            base_dir: env::current_dir().unwrap_or(PathBuf::from("./")),
            host: "127.0.0.1".to_string(),
            port: 8080,
            log_level: log::Level::Info,
            workers: 2,
        }
    }
}

impl ServerConfigs {
    pub fn builder() -> ServerConfigsBuilder {
        ServerConfigsBuilder {
            base_dir: None,
            host: None,
            port: None,
            log_level: None,
            workers: None,
        }
    }

    pub fn from_cli_args() -> Self {
        let matches = command!()
            .arg(
                arg!([base_dir] "Optional base directory to serve. Current working directory by default.")
                    .required(false)
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(
                arg!(-p --port <PORT> "Sets custom port. Default = 8080")
                    .required(false)
                    .value_parser(value_parser!(u16)),
            )
            .arg(
                arg!(-H --host <HOST> "Sets custom host. Default = 127.0.0.1")
                    .required(false)
                    .value_parser(value_parser!(String)),
            )
            .arg(
                arg!(-l --loglevel <LOGLEVEL> "Sets log level. Default = info")
                    .required(false)
                    .value_parser(value_parser!(String)),
            )
            .arg(
                arg!(-w --workers <WORKERS> "Sets number of worker threads. Default = 2")
                    .required(false)
                    .value_parser(value_parser!(usize)),
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
        if let Some(&workers) = matches.get_one::<usize>("workers") {
            configs_builder.workers(workers);
        }

        configs_builder.build()
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfigsBuilder {
    base_dir: Option<PathBuf>,
    host: Option<String>,
    port: Option<u16>,
    log_level: Option<log::Level>,
    workers: Option<usize>,
}

impl ServerConfigsBuilder {
    pub fn base_dir(&mut self, path: &PathBuf) -> &Self {
        self.base_dir = Some(path.to_owned());
        self
    }

    pub fn host(&mut self, host: &str) -> &Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn port(&mut self, port: u16) -> &Self {
        self.port = Some(port);
        self
    }

    pub fn log_level(&mut self, log_level: &str) -> &Self {
        let log_level = match log_level {
            "error" => log::Level::Error,
            "warn" => log::Level::Warn,
            "debug" => log::Level::Debug,
            "trace" => log::Level::Trace,
            _ => log::Level::Info,
        };
        self.log_level = Some(log_level);
        self
    }

    pub fn workers(&mut self, workers: usize) -> &Self {
        self.workers = Some(workers);
        self
    }

    pub fn build(mut self) -> ServerConfigs {
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
        if let Some(workers) = self.workers.take() {
            config.workers = workers;
        }

        config
    }
}
