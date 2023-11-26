mod domain;
mod routes;
mod templating;

use actix_web::{web, App, HttpServer};
use domain::repository::{AppState, Date, VecRepo};
use routes::dates::dates_service;
use routes::index::index;
use std::sync::Mutex;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let state = AppState::new(Mutex::new(Box::new(VecRepo::new(vec![
        Date::new("Italian Resturant".into()),
        Date::new("Sexy Times".into()),
    ]))));
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(state.clone())
            .service(index)
            .service(web::scope("/dates").configure(dates_service))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await?;
    Ok(())
}
