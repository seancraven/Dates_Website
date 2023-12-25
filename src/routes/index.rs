use crate::domain::repository::AppState;
use crate::domain::{dates::Date, repository::ExpansionCache};
use crate::routes::dates_service::render_buttons;
use actix_web::{get, web::Data, web::Path, HttpResponse, Responder};
use anyhow::anyhow;
use tera::{Context, Tera};
use uuid::Uuid;

#[get("/{user_id}")]
pub async fn index(app_state: Data<AppState>, user_id: Path<Uuid>) -> impl Responder {
    app_state.repo.app_state.cache.reset(&user_id);
    HttpResponse::Ok().body(
        template_load(
            app_state.repo.get_all(&user_id).await,
            &app_state.cache,
            &user_id,
        )
        .unwrap(),
    )
}
fn template_load(
    dates: Vec<Date>,
    cache: &ExpansionCache,
    user_id: &Uuid,
) -> anyhow::Result<String> {
    let mut ctx = Context::new();
    let buttons = render_buttons(dates, cache, user_id)?;
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
