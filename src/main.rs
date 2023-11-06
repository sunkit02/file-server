use actix_web;
use configs::ServerConfigs;

mod configs;
mod file_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configs = ServerConfigs::from_cli_args();

    file_server::start(configs).await
}
