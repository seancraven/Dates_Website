use crate::domain::dates::Date;
use crate::domain::repository::AppState;
use actix_web::web::ServiceConfig;
use actix_web::{post, web};
use actix_web::{HttpResponse, Responder};
use anyhow::anyhow;
use std::collections::HashMap;
use std::fs::read;
use tera::{self, Context, Tera};

pub fn dates_service(cfg: &mut ServiceConfig) {
    cfg.service(date_count_increment)
        .service(date_count_decrement)
        .service(add_new_date)
        .service(date_remove);
}

#[post("{date_info}/increment")]
async fn date_count_increment(
    date_info: web::Path<uuid::Uuid>,
    app_state: AppState,
) -> impl Responder {
    let date_id = &date_info;
    tracing::info!("Increment pushed on: {}", &date_id);
    match app_state.increment_date_count(date_id).await {
        Ok(_) => HttpResponse::Ok().body(render_buttons(app_state.get_all().await).unwrap()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
#[post("{date_info}/decrement")]
async fn date_count_decrement(
    date_info: web::Path<uuid::Uuid>,
    app_state: AppState,
) -> impl Responder {
    let date_id = &date_info;
    tracing::info!("Decrement pushed on: {}", &date_id);
    match app_state.decrement_date_count(date_id).await {
        Ok(_) => HttpResponse::Ok().body(render_buttons(app_state.get_all().await).unwrap()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
#[post("{date_info}/remove")]
async fn date_remove(date_info: web::Path<uuid::Uuid>, app_state: AppState) -> impl Responder {
    let date_id = &date_info;
    tracing::info!("Remove pushed on: {}", &date_id);
    match app_state.remove(date_id).await {
        Ok(_) => HttpResponse::Ok().body(render_buttons(app_state.get_all().await).unwrap()),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
#[post("new_date")]
async fn add_new_date(
    new_date: web::Form<HashMap<String, String>>,
    app_state: AppState,
) -> impl Responder {
    tracing::info!(
        "Date added: {}",
        new_date.get("new_date").unwrap_or(&String::from("Failed"))
    );
    if new_date.get("new_date").unwrap().is_empty() {
        return HttpResponse::Forbidden().finish();
    }
    app_state
        .add(Date::new(new_date.get("new_date").unwrap().clone()))
        .await
        .unwrap();
    HttpResponse::Ok().body(render_buttons(app_state.get_all().await).unwrap())
}
fn render_buttons(dates: Vec<Date>) -> anyhow::Result<String> {
    let mut ctx = Context::new();
    ctx.insert("dates", &dates);
    Tera::one_off(
        std::str::from_utf8(&read("./pages/button.html")?)?,
        &ctx,
        false,
    )
    .map_err(|_| anyhow!("Failed to render template"))
}
