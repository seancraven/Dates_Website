mod domain;
mod routes;
mod templating;

use actix_web::web;
use actix_web::web::ServiceConfig;
use domain::postgres_repository::PgRepo;
use domain::repository::AppState;
use routes::dates_service::dates_service;
use routes::index::index;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::PgPool;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    sqlx::migrate!().run(&pool).await.unwrap();
    let state = AppState::new_in_web_data(Box::new(PgRepo { pool }));
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("/dates")
                .wrap(actix_web::middleware::Logger::default())
                .app_data(state.clone())
                .service(index)
                .configure(dates_service),
        );
    };
    Ok(config.into())
}
