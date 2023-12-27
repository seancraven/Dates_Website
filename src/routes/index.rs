use std::fs;

use crate::domain::repository::{AppState, MissingUserError};
use crate::domain::{dates::Date, repository::ExpansionCache};
use crate::routes::dates_service::render_dates;
use actix_web::{get, web::Data, web::Path, HttpResponse, Responder};
use anyhow::anyhow;
use log::debug;
use tera::{Context, Tera};
use uuid::Uuid;

pub fn unauthorized() -> HttpResponse {
    HttpResponse::Unauthorized().body(fs::read_to_string("./pages/disallowed.html").unwrap())
}

#[get("/{user_id}")]
pub async fn index(app_state: Data<AppState>, user_id: Path<Uuid>) -> impl Responder {
    if !app_state.repo.check_user_has_access(&user_id).await {
        log::info!("Unauthorized user {:?} attempted access", user_id);
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
#[get("/googleb0081feae6701197.html")]
async fn search_verification() -> impl Responder {
    HttpResponse::Ok().body(fs::read_to_string("./pages/googleb0081feae6701197.html").unwrap())
}

#[cfg(test)]
mod tests {
    use actix_web::App;
    use uuid::Uuid;

    use crate::domain::{
        dates::Date,
        repository::{AppState, Repository, VecRepo},
    };

    use super::index;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
    #[actix_web::test]
    async fn test_index() {
        let repo = VecRepo::new();
        let user_id = Uuid::new_v4();
        repo.add(Date::new("test"), user_id).await.unwrap();
        let state = AppState::new(Box::new(repo));
        let app = actix_web::test::init_service(App::new().app_data(state).service(index)).await;
        let req = actix_web::test::TestRequest::get()
            .uri(&format!("/{}", user_id))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
