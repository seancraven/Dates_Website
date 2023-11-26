mod domain;
mod routes;
mod templating;

use actix_web::web;
use actix_web::web::ServiceConfig;
use domain::repository::{AppState, VecRepo};
use routes::dates::dates_service;
use routes::index::index;
use shuttle_actix_web::ShuttleActixWeb;
use std::sync::Mutex;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let state = AppState::new(Mutex::new(Box::new(VecRepo::new(vec![
        // Date::new("Italian Resturant".into()),
        // Date::new("Sexy Times".into()),
    ]))));
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("/distributed_dates")
                .wrap(actix_web::middleware::Logger::default())
                .app_data(state.clone())
                .service(index)
                .service(web::scope("/dates").configure(dates_service)),
        );
    };
    Ok(config.into())
}
