use actix_web::web;

pub mod handlers;
pub mod models;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(handlers::serve_static_file)
        .service(handlers::dir_structure);
}
