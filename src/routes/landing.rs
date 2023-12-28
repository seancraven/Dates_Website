use std::fs;

use crate::auth::user::UnauthorizedUser;
use crate::domain::repository::AppState;
use actix_web::{get, web::Data, HttpResponse, Responder};
use tera::{Context, Tera};
use tracing::{error, info};

pub fn unauthorized() -> HttpResponse {
    HttpResponse::Unauthorized().body(fs::read_to_string("./pages/disallowed.html").unwrap())
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

#[get("/login/get_new_user")]
async fn create_user(app_state: Data<AppState>) -> HttpResponse {
    // TODO: Hack to get working fast.
    let user_info = UnauthorizedUser {
        email: String::from("dave@dave.com"),
        username: String::from("dave"),
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
            HttpResponse::Ok().body(
                Tera::one_off(
                    &fs::read_to_string("./pages/user.html").unwrap(),
                    &ctx,
                    false,
                )
                .unwrap(),
            )
        }
        Err(e) => {
            error!("{:?}", e);
            error!("Failed to create user: {:?}", user_info);

            unauthorized()
        }
    }
}
