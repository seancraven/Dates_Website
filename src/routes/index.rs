use crate::domain::dates::Date;
use crate::domain::repository::AppState;
use actix_web::{get, HttpResponse, Responder};
use tera::{Context, Tera};

#[get("")]
pub async fn index(repo: AppState) -> impl Responder {
    log::info!("Serving index page");
    HttpResponse::Ok().body(template_load(repo.get_all().await).unwrap())
}
fn template_load(dates: Vec<Date>) -> anyhow::Result<String> {
    let mut ctx = Context::new();
    ctx.insert("dates", &dates);
    let tera = Tera::new("./pages/*.html").unwrap();
    let html = tera.render("index.html", &ctx).unwrap();
    Ok(html)
}
