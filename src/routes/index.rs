use crate::domain::repository::AppState;
use crate::domain::{dates::Date, repository::ExpansionCache};
use crate::routes::dates_service::render_buttons;
use actix_web::{get, web::Data, HttpResponse, Responder};
use anyhow::anyhow;
use tera::{Context, Tera};

#[get("")]
pub async fn index(app_state: Data<AppState>) -> impl Responder {
    log::info!("Serving index page");
    app_state.cache.reset();
    HttpResponse::Ok()
        .body(template_load(app_state.repo.get_all().await, &app_state.cache).unwrap())
}
fn template_load(dates: Vec<Date>, cache: &ExpansionCache) -> anyhow::Result<String> {
    let mut ctx = Context::new();
    let buttons = render_buttons(dates, cache)?;
    ctx.insert("buttons", &buttons);
    Tera::one_off(&std::fs::read_to_string("./pages/index.html")?, &ctx, false)
        .map_err(|e| anyhow!(e))
}

#[get("/")]
async fn landing() -> impl Responder {
    HttpResponse::Ok().body(std::fs::read_to_string("./pages/landing.html").unwrap())
}
#[get("/googleb0081feae6701197.html")]
async fn search_verification() -> impl Responder {
    HttpResponse::Ok().body(std::fs::read_to_string("./pages/googleb0081feae6701197.html").unwrap())
}
