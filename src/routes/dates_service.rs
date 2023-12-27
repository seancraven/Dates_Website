use super::index::unauthorized;
use crate::domain::dates::Date;
use crate::domain::dates::Status;
use crate::domain::repository::InsertDateError;
use crate::domain::repository::{AppState, ExpansionCache};
use actix_web::web::ServiceConfig;
use actix_web::{delete, HttpResponse, Responder};
use actix_web::{get, post, web, web::Data};
use anyhow::anyhow;
use chrono::{Local, NaiveDateTime};
use std::collections::HashMap;
use std::fs::read;
use tera::{self, Context, Tera};
use tracing::error;
use uuid::Uuid;

pub fn dates_service(cfg: &mut ServiceConfig) {
    cfg.service(date_count_increment)
        .service(date_count_decrement)
        .service(date_remove)
        .service(date_expand)
        .service(date_collapse)
        .service(edit_description)
        .service(get_description)
        .service(update_description)
        .service(add_new_date);
}

#[post("{user_id}/{date_id}/increment")]
async fn date_count_increment(
    date_id: web::Path<Uuid>,
    user_id: web::Data<Uuid>,
    app_state: Data<AppState>,
) -> impl Responder {
    let date_id = &date_id;
    tracing::info!("Increment pushed on: {}", &date_id);
    match app_state.repo.increment_date_count(date_id, &user_id).await {
        Ok(_) => HttpResponse::Ok().body(
            render_dates(
                app_state.repo.get_all(&user_id).await,
                &app_state.cache,
                &user_id,
            )
            .unwrap(),
        ),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
#[post("{user_id}/{date_id}/decrement")]
async fn date_count_decrement(
    date_id: web::Path<Uuid>,
    user_id: web::Data<Uuid>,
    app_state: Data<AppState>,
) -> impl Responder {
    let date_id = &date_id;
    tracing::info!("Decrement pushed on: {}", &date_id);
    match app_state.repo.decrement_date_count(date_id, &user_id).await {
        Ok(_) => HttpResponse::Ok().body(
            render_dates(
                app_state.repo.get_all(&user_id).await,
                &app_state.cache,
                &user_id,
            )
            .unwrap(),
        ),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
#[post("{user_id}/{date_id}/remove")]
async fn date_remove(
    date_id: web::Path<Uuid>,
    user_id: web::Data<Uuid>,
    app_state: Data<AppState>,
) -> impl Responder {
    let date_id = &date_id;
    tracing::info!("Collapse pushed on: {}", &date_id);
    match app_state.repo.remove(date_id, &user_id).await {
        Ok(_) => HttpResponse::Ok().body(
            render_dates(
                app_state.repo.get_all(&user_id).await,
                &app_state.cache,
                &user_id,
            )
            .unwrap(),
        ),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
#[get("{user_id}/{date_id}")]
async fn date_expand(
    date_id: web::Path<Uuid>,
    user_id: web::Path<Uuid>,
    app_state: Data<AppState>,
) -> impl Responder {
    tracing::info!("Expand pushed on: {}", date_id);
    match app_state.repo.get(&date_id, &user_id).await {
        Some(date) => {
            let mut ctx = Context::new();
            ctx.insert("description", &render_description(&date).unwrap());
            ctx.insert("date", &date);
            let tera = Tera::new("./pages/button/*.html").unwrap();
            match tera.render("button_expanded.html", &ctx) {
                Ok(resp) => {
                    if app_state.cache.add(*date_id, &user_id).is_err() {
                        return unauthorized();
                    };
                    HttpResponse::Ok().body(resp)
                }
                Err(e) => {
                    error!("{}", e.to_string());
                    HttpResponse::InternalServerError().body(e.to_string())
                }
            }
        }
        None => HttpResponse::InternalServerError().body("Date not found"),
    }
}
#[post("{user_id}/{date_id}")]
async fn date_collapse(
    date_id: web::Path<Uuid>,
    user_id: web::Path<Uuid>,
    app_state: Data<AppState>,
) -> impl Responder {
    let date_id = &date_id;
    tracing::info!("Collapse pushed on: {}", &date_id);
    match app_state.repo.get(date_id, &user_id).await {
        Some(date) => {
            let mut ctx = Context::new();
            ctx.insert("date", &date);
            let tera = Tera::new("./pages/button/*.html").unwrap();
            match tera.render("button_collapsed.html", &ctx) {
                Ok(resp) => {
                    if app_state.cache.remove(date_id, &user_id).is_err() {
                        return unauthorized();
                    }
                    HttpResponse::Ok().body(resp)
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        None => HttpResponse::InternalServerError().body("Date not found"),
    }
}
#[post("{user_id}/{date_id}/description")]
async fn update_description(
    mut map: web::Form<HashMap<String, String>>,
    ids: web::Path<(Uuid, Uuid)>,
    app_state: Data<AppState>,
) -> impl Responder {
    let (user_id, date_id) = *ids;
    let Some(mut date) = app_state.repo.get(&date_id, &user_id).await else {
        return HttpResponse::InternalServerError().body("Date not found");
    };
    let hrs = map.remove("time").unwrap();
    let day = map.remove("day").unwrap();
    if let Ok(naive_date_time) =
        NaiveDateTime::parse_from_str(&format!("{} {}", hrs, day), "%H:%M %Y-%m-%d")
    {
        tracing::debug!("Date time updated: {}:{}", hrs, day);
        date.description.day = Some(naive_date_time.and_local_timezone(Local).unwrap());
    } else if hrs.is_empty() || day.is_empty() {
        error!("Cant't parse date {:?} from {} {}", date, hrs, day);
        return HttpResponse::Forbidden().body("Cant parse date");
    };
    tracing::debug!(
        "Date description updated: {}",
        map.get("description_text").unwrap()
    );
    date.description.text = map.remove("description_text").unwrap();
    app_state.repo.update(date.clone(), &user_id).await.unwrap();
    HttpResponse::Ok().body(render_description(&date).unwrap())
}
#[delete("{user_id}/{date_id}/description")]
async fn edit_description(
    ids: web::Path<(Uuid, Uuid)>,
    app_state: Data<AppState>,
) -> impl Responder {
    let (user_id, date_id) = *ids;
    match app_state.repo.get(&date_id, &user_id).await {
        Some(date) => HttpResponse::Ok().body(
            render_editable_description(&date)
                .map_err(|e| {
                    error!("{:?}", e);
                    e
                })
                .unwrap(),
        ),
        None => HttpResponse::InternalServerError().body("Date not found"),
    }
}
#[get("{user_id}/{date_id}/description")]
async fn get_description(
    ids: web::Path<(Uuid, Uuid)>,
    app_state: Data<AppState>,
) -> impl Responder {
    let (user_id, date_id) = *ids;
    match app_state.repo.get(&date_id, &user_id).await {
        Some(date) => HttpResponse::Ok().body(
            render_description(&date)
                .map_err(|e| {
                    error!("{:?}", e);
                    e
                })
                .unwrap(),
        ),
        None => HttpResponse::InternalServerError().body("Date not found"),
    }
}
#[post("{user_id}/new_date")]
async fn add_new_date(
    mut new_date: web::Form<HashMap<String, String>>,
    user_id: web::Path<Uuid>,
    app_state: Data<AppState>,
) -> impl Responder {
    if !app_state.repo.check_user_has_access(&user_id).await {
        return unauthorized();
    }
    if let Some(new_date_name) = new_date.remove("new_date") {
        if new_date_name.is_empty() {
            return HttpResponse::Forbidden().finish();
        };
        match app_state.repo.add(Date::new(new_date_name), *user_id).await {
            Ok(_) => (),
            Err(e) => {
                error!("{:?}", e);
                match e {
                    InsertDateError::QueryError => {
                        return HttpResponse::InternalServerError().finish()
                    }
                    InsertDateError::GroupMembershipError => return unauthorized(),
                }
            }
        }
        HttpResponse::Ok().body(
            render_dates(
                app_state.repo.get_all(&user_id).await,
                &app_state.cache,
                &user_id,
            )
            .map_err(|e| {
                error!("{:?}", e);
                e
            })
            .unwrap(),
        )
    } else {
        HttpResponse::Forbidden().finish()
    }
}

// TODO: Make this only take in a repository and a user id.
// Further I think the error logging should be morved to the caller.
//
//
/// Render the list of current dates.
/// Keeps dates open that have been expanded by the user.
///  
///
/// * `dates`: List of dates to render.
/// * `cache`: A cache of which dates a user has expanded.
/// * `user_id`: The user id to render the dates for.
pub fn render_dates(
    dates: Vec<Date>,
    cache: &ExpansionCache,
    user_id: &Uuid,
) -> anyhow::Result<String> {
    let mut rendered_dates = vec![];
    for date in dates {
        let mut ctx = Context::new();
        ctx.insert("date", &date);
        ctx.insert("user_id", user_id);
        let tera = Tera::new("./pages/button/*.html")?;
        if cache.contains(&date.id, user_id).unwrap_or(false) {
            ctx.insert("description", &render_description(&date)?);
            rendered_dates.push(
                tera.render("button_expanded.html", &ctx)
                    .map_err(|e| anyhow!(e))?,
            );
        } else {
            rendered_dates.push(
                tera.render("button_collapsed.html", &ctx)
                    .map_err(|e| anyhow!(e))?,
            );
        }
    }
    let mut ctx = Context::new();
    ctx.insert("dates", &rendered_dates);
    ctx.insert("user_id", user_id);
    Tera::one_off(
        std::str::from_utf8(&read("./pages/buttons.html")?)?,
        &ctx,
        false,
    )
    .map_err(|e| anyhow!(e))
}

fn render_description(date: &Date) -> anyhow::Result<String> {
    let mut ctx = Context::new();
    let date_str = date.description.render_date();
    let status_str = date.description.render_status();
    let color = String::from(match date.description.status {
        Status::Suggested => "bg-cyan-50",
        Status::Approved => "bg-green-50",
        Status::Rejected => "bg-red-50",
    });
    ctx.insert("date", &date);
    ctx.insert("status", &status_str);
    ctx.insert("status_color", &color);
    if date.description.text.is_empty() {
        ctx.insert("text", "Enter a description!");
    } else {
        ctx.insert("text", &date.description.text);
    }
    ctx.insert("date_time", &date_str);
    Tera::one_off(
        std::str::from_utf8(&read("./pages/button/description/description.html")?)?,
        &ctx,
        false,
    )
    .map_err(|e| anyhow!(e))
}
fn render_editable_description(date: &Date) -> anyhow::Result<String> {
    let mut ctx = Context::new();
    let date_str = date.description.render_date();
    let status_str = date.description.render_status();
    let color = String::from(match date.description.status {
        Status::Suggested => "bg-cyan-50",
        Status::Approved => "bg-green-50",
        Status::Rejected => "bg-red-50",
    });
    ctx.insert("date", &date);
    ctx.insert("status", &status_str);
    ctx.insert("status_color", &color);
    ctx.insert("date_time", &date_str);
    Tera::one_off(
        std::str::from_utf8(&read("./pages/button/description/description_form.html")?)?,
        &ctx,
        false,
    )
    .map_err(|e| anyhow!(e))
}
