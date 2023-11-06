use actix_web::web;

pub mod handlers;
pub mod models;
pub mod templates;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/").service(handlers::home_page));
}
