use std::fs;

use crate::auth::user::{AuthorizedUser, NoGroupUser, UnauthorizedUser};
use crate::domain::repository::AppState;
use crate::routes::dates_service::render_dates;
use actix_web::error::ErrorInternalServerError;
use actix_web::Result;
use actix_web::{
    get, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use tera::{Context, Tera};
use tracing::{error, info};
use uuid::Uuid;

pub fn unauthorized() -> Result<HttpResponse> {
    Ok(HttpResponse::Unauthorized()
        .body(fs::read_to_string("./pages/disallowed.html").map_err(ErrorInternalServerError)?))
}

#[get("/")]
async fn landing() -> Result<impl Responder> {
    Ok(HttpResponse::Ok()
        .body(fs::read_to_string("./pages/landing.html").map_err(ErrorInternalServerError)?))
}
#[post("/")]
async fn login_register(
    app_state: Data<AppState>,
    form: web::Form<UnauthorizedUser>,
) -> Result<impl Responder> {
    let Ok(user) = app_state.repo.validate_user(&form).await else {
        let body = fs::read_to_string("./pages/landing.html")?;
        return Ok(HttpResponse::Unauthorized().body(body));
    };

    match user {
        AuthorizedUser::GroupUser(u) => {
            let dates = app_state.repo.get_all(&u.id).await;
            let dates =
                render_dates(dates, &app_state.cache, &u.id).map_err(ErrorInternalServerError)?;
            Ok(HttpResponse::Ok().body(dates))
        }
        AuthorizedUser::NoGroupUser(_) => {
            let body = fs::read_to_string("./pages/user.html")?;
            Ok(HttpResponse::Ok().body(body))
        }
    }
}
// TODO: This is a dummy version of the login page.
#[get("/login")]
async fn dummy_login() -> Result<impl Responder> {
    Ok(HttpResponse::Ok()
        .body(fs::read_to_string("./pages/dummy_login.html").map_err(ErrorInternalServerError)?))
}
#[get("/googleb0081feae6701197.html")]
async fn search_verification() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().body(
        fs::read_to_string("./pages/googleb0081feae6701197.html")
            .map_err(ErrorInternalServerError)?,
    ))
}

#[get("/login/get_new_user")]
async fn create_user(app_state: Data<AppState>) -> Result<HttpResponse> {
    // TODO: Hack to get working fast.
    let user_info = NoGroupUser {
        id: Uuid::new_v4(),
        email: String::from("dave@dave.com"),
    };
    info!("Creating user: {:?}", &user_info);

    match app_state
        .repo
        .create_user_and_group(user_info.clone())
        .await
    {
        Ok(user) => {
            info!("Created User: {:?}", user);
            let mut ctx = Context::new();
            ctx.insert("user", &user);
            Ok(HttpResponse::Ok().body(
                Tera::one_off(
                    &fs::read_to_string("./pages/user.html").map_err(ErrorInternalServerError)?,
                    &ctx,
                    false,
                )
                .map_err(ErrorInternalServerError)?,
            ))
        }
        Err(e) => {
            error!("{:?}", e);
            error!("Failed to create user: {:?}", user_info);
            unauthorized()
        }
    }
}
