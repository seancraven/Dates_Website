use std::collections::HashMap;
use std::fs;

use crate::auth::user::{AuthorizedUser, UnauthorizedUser};
use crate::backend::postgres::PgRepo;
use crate::domain::repository::AppState;
use crate::routes::dates_service::dates_service;
use crate::routes::dates_service::render_dates;
use actix_web::error::ErrorInternalServerError;
use actix_web::middleware::Logger;
use actix_web::Result;
use actix_web::{
    get, post,
    web::{self, Data, ServiceConfig},
    HttpResponse, Responder,
};
use secrecy::Secret;
use sqlx::PgPool;
use tera::{Context, Tera};

pub struct DatesService {
    pool: PgPool,
}
impl DatesService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub fn service_configuration(self, cfg: &mut ServiceConfig) {
        cfg.app_data(AppState::new_in_web_data(Box::new(PgRepo {
            pool: self.pool,
        })))
        .service(
            web::scope("")
                .wrap(Logger::default())
                .service(landing)
                .service(login)
                .service(register)
                .service(search_verification)
                .service(
                    web::scope("/dates")
                        .wrap(Logger::default())
                        .configure(dates_service),
                ),
        );
    }
}

pub fn unauthorized() -> Result<HttpResponse> {
    Ok(HttpResponse::Unauthorized()
        .body(fs::read_to_string("./pages/disallowed.html").map_err(ErrorInternalServerError)?))
}

#[get("/")]
pub async fn landing() -> Result<impl Responder> {
    Ok(HttpResponse::Ok()
        .body(fs::read_to_string("./pages/landing.html").map_err(ErrorInternalServerError)?))
}
#[post("/login")]
async fn login(
    app_state: Data<AppState>,
    mut form: web::Form<HashMap<String, String>>,
) -> Result<impl Responder> {
    let u_user = UnauthorizedUser {
        email: form.remove("email").unwrap(),
        password: Secret::new(form.remove("password").unwrap()),
    };
    let Ok(user) = app_state.repo.validate_user(&u_user).await else {
        return Ok(HttpResponse::Unauthorized().body("Not a valid user."));
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
#[post("/register")]
async fn register(
    app_state: Data<AppState>,
    mut form: web::Form<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let u_user = UnauthorizedUser {
        email: form.remove("email").unwrap(),
        password: Secret::new(form.remove("password").unwrap()),
    };
    let user = app_state
        .repo
        .create_authorized_user(u_user)
        .await
        .map_err(ErrorInternalServerError)?;
    let mut ctx = Context::new();
    ctx.insert("user_id", &user.id);
    ctx.insert("user_email", &user.email);
    let body = Tera::one_off(&fs::read_to_string("./pages/user.html")?, &ctx, false)
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(body))
}
#[get("googleb0081feae6701197.html")]
async fn search_verification() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().body(
        fs::read_to_string("./pages/googleb0081feae6701197.html")
            .map_err(ErrorInternalServerError)?,
    ))
}
