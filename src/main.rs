mod domain;
mod routes;
mod templating;

use actix_web::web;
use actix_web::web::ServiceConfig;
use domain::repository::{AppState, VecRepo};
use routes::dates::dates_service;
use routes::index::index;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::PgPool;
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    sqlx::migrate!().run(&pool).await.unwrap();
    let state = AppState::new(Box::new(VecRepo::new(vec![])));
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
