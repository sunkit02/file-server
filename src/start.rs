use std::env;

use actix_web::{web::Data, App, HttpServer};
use log::info;

use crate::{configs::ServerConfigs, file_manager, file_server};

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
            .configure(file_server::config)
            .configure(file_manager::config)
    })
    .workers(2)
    .bind((configs.host, configs.port))?
    .run()
    .await
}
