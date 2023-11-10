use configs::ServerConfigs;

mod configs;
mod file_manager;
mod file_server;
mod start;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configs = ServerConfigs::from_cli_args();

    start::start(configs).await
}
