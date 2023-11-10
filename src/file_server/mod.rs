use actix_web::web;

pub mod handlers;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(handlers::health_check)
        .service(handlers::serve_static_file)
        .service(handlers::dir_structure);
}
