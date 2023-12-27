use std::fs;

use crate::auth::user::UnauthorizedUser;
use crate::domain::repository::{AppState, MissingUserError};
use crate::domain::{dates::Date, repository::ExpansionCache};
use crate::routes::dates_service::render_dates;
use actix_web::web::Form;
use actix_web::{get, web::Data, web::Path, HttpResponse, Responder};
use anyhow::anyhow;
use tera::{Context, Tera};
use tracing::{debug, error, info};
use uuid::Uuid;

pub fn unauthorized() -> HttpResponse {
    HttpResponse::Unauthorized().body(fs::read_to_string("./pages/disallowed.html").unwrap())
}

#[get("/{user_id}")]
pub async fn index(app_state: Data<AppState>, user_id: Path<Uuid>) -> impl Responder {
    if !app_state.repo.check_user_has_access(&user_id).await {
        info!("Unauthorized user {:?} attempted access", user_id);
        return unauthorized();
    }
    if app_state.cache.reset(&user_id).is_err() {
        debug!("Cache doesn't contain {:?}", user_id);
    };
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
    let buttons = render_dates(dates, cache, user_id)?;
    ctx.insert("buttons", &buttons);
    Tera::one_off(&fs::read_to_string("./pages/index.html")?, &ctx, false).map_err(|e| anyhow!(e))
}

#[get("/")]
async fn landing() -> impl Responder {
    HttpResponse::Ok().body(fs::read_to_string("./pages/landing.html").unwrap())
}
// TODO: This is a dummy version of the login page.
#[get("/login")]
async fn dummy_login() -> impl Responder {
    HttpResponse::Ok().body(fs::read_to_string("./pages/dummy_login.html").unwrap())
}
#[get("/googleb0081feae6701197.html")]
async fn search_verification() -> impl Responder {
    HttpResponse::Ok().body(fs::read_to_string("./pages/googleb0081feae6701197.html").unwrap())
}

// #[get("/login/get_new_user")]
// async fn create_user(app_state: Data<AppState>, user_info: Form<UnauthorizedUser>) -> HttpResponse {
//     let Ok(user) = user_info
//         .into_inner()
//         .create_user_and_group(app_state)
//         .await
//         .map_err(|e| {
//             error!("{:?}", e);
//             e
//         });
// }
