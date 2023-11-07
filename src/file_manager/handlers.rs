use actix_web::{get, http::header::ContentType, HttpResponse, Responder};
use log::info;

#[get("/")]
pub async fn home_page() -> impl Responder {
    info!("Getting home page");
    HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body("<h1>Home Page</h1>")
}

#[get("/favicon.ico")]
pub async fn favicon() -> impl Responder {
    let favicon_bytes = include_bytes!("../../public/folder.svg").as_slice();
    HttpResponse::Ok()
        .insert_header(("Content-Type", "image/svg+xml"))
        .body(favicon_bytes)
}

