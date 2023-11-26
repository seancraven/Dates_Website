use crate::domain::repository::AppState;
use actix_web::{get, HttpResponse, Responder};
use tera::{Context, Tera};

#[get("")]
pub async fn index(repo: AppState) -> impl Responder {
    log::info!("Serving index page");
    HttpResponse::Ok().body(template_load(repo).expect("Templating failed."))
}
pub fn template_load(repo: AppState) -> anyhow::Result<String> {
    let mut ctx = Context::new();
    ctx.insert("dates", &repo.lock().unwrap().get_all());
    let tera = Tera::new("./pages/*.html").unwrap();
    let html = tera.render("index.html", &ctx).unwrap();
    Ok(html)
}
