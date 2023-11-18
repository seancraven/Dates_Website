use actix_web::{get, HttpRequest, HttpResponse, Responder};
use std::fs::read;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(read("./pages/index.html").expect("failed to get index"))
}
