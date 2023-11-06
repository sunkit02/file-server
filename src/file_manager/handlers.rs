use actix_web::{get, http::header::ContentType, HttpResponse, Responder};

#[get("")]
pub async fn home_page() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body("<h1>Home Page</h1>")
}
