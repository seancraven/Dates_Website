use std::collections::HashMap;
use std::fs;

use crate::auth::user::{AuthorizedUser, UnAuthorizedUser, UnRegisteredUser, UserValidationError};
use crate::backend::postgres::PgRepo;
use crate::domain::repository::AppState;
use crate::email::{authenticate_by_email, EmailClient};
use crate::routes::dates_service::{date_page_inner, dates_service};
use actix_web::error::{
    ErrorForbidden, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized,
};
use actix_web::middleware::Logger;
use actix_web::Result;
use actix_web::{
    get, post,
    web::{self, Data, Path, ServiceConfig},
    HttpResponse, Responder,
};

use sqlx::PgPool;
use tera::{Context, Tera};
use uuid::Uuid;

pub struct MainService {
    pool: PgPool,
    email_client: EmailClient,
}
impl MainService {
    pub fn new(pool: PgPool, email_client: EmailClient) -> Self {
        Self { pool, email_client }
    }
    pub fn service_configuration(self, cfg: &mut ServiceConfig) {
        cfg.app_data(AppState::new_in_web_data(
            Box::new(PgRepo { pool: self.pool }),
            self.email_client,
        ))
        .service(
            web::scope("")
                .wrap(Logger::default())
                .service(web::redirect("", "/"))
                .service(landing)
                .service(login)
                .service(register)
                .service(create_group)
                .service(search_verification)
                .service(authenticate_by_email)
                .service(web::scope("/dates").configure(dates_service)),
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
) -> Result<HttpResponse> {
    let u_user = UnAuthorizedUser::new(
        form.remove("email").unwrap(),
        form.remove("password").unwrap(),
    );
    let user = app_state
        .repo
        .validate_user(&u_user)
        .await
        .map_err(|e| match e {
            UserValidationError::PasswordError(_) => ErrorUnauthorized("User Password failed."),
            UserValidationError::RegistrationError(e) => ErrorNotFound(e),
            _ => ErrorInternalServerError("Server Error."),
        })?;

    Ok(HttpResponse::Ok().body(render_user_page(user)?))
}
fn render_user_page(user: AuthorizedUser) -> Result<String> {
    let mut ctx = Context::new();
    if let Some(group) = user.group() {
        ctx.insert("group", &group);
        ctx.insert("user_uri", &format!("/dates/{:?}", &user.id()));
    }
    ctx.insert("user_id", &user.id());
    ctx.insert("user_email", &user.email());
    ctx.insert("method", "post");
    ctx.insert("uri", &format!("{:?}/create_group", &user.id()));
    Tera::one_off(&fs::read_to_string("./pages/user.html")?, &ctx, false)
        .map_err(ErrorInternalServerError)
}
#[post("/register")]
async fn register(
    app_state: Data<AppState>,
    mut form: web::Form<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let u_user = UnRegisteredUser::new(
        form.remove("email").unwrap(),
        form.remove("password").unwrap(),
    );
    app_state
        .repo
        .register_user(u_user.clone())
        .await
        .map_err(ErrorInternalServerError)?;
    app_state
        .email_client
        .send_auth_email(&u_user.email)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body("Check your email for a link to activate your account."))
}

#[post("/authorize/{user_id}")]
async fn authorize(app_state: Data<AppState>, user_id: Path<Uuid>) -> Result<HttpResponse> {
    let user = app_state
        .repo
        .activate_user(&user_id)
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(render_user_page(AuthorizedUser::NoGroupUser(user))?))
}
#[post("/{user_id}/create_group")]
async fn create_group(app_state: Data<AppState>, user_id: Path<Uuid>) -> Result<HttpResponse> {
    let group = app_state
        .repo
        .create_group()
        .await
        .map_err(ErrorInternalServerError)?;
    let AuthorizedUser::NoGroupUser(user) = app_state
        .repo
        .get_user(&user_id)
        .await
        .map_err(ErrorInternalServerError)?
    else {
        return Err(ErrorForbidden("Cant Change Group."));
    };
    let group_user = app_state
        .repo
        .add_user_to_group(user, group)
        .await
        .map_err(ErrorInternalServerError)?;
    date_page_inner(app_state.into_inner(), group_user.user_id).await
}

#[get("googleb0081feae6701197.html")]
async fn search_verification() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().body(
        fs::read_to_string("./pages/googleb0081feae6701197.html")
            .map_err(ErrorInternalServerError)?,
    ))
}
