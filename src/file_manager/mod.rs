use actix_web::web;

pub mod handlers;
pub mod models;
pub mod templates;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(handlers::home_page)
        .service(handlers::favicon)
        .service(handlers::dir_structure);
}
