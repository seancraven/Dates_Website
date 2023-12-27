use crate::routes::dates_service;
use actix_web::web;
use actix_web::web::ServiceConfig;
use dates::domain::postgres_repository::PgRepo;
use dates::domain::repository::AppState;
use dates::routes::index::{index, landing, search_verification};
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::PgPool;

// postgres://postgres:postgres@localhost:17972/postgres
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    sqlx::migrate!().run(&pool).await.unwrap();
    // let state = web::Data::new(AppState::new(Box::new(VecRepo::new())));
    let state = AppState::new_in_web_data(Box::new(PgRepo { pool }));
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("/dates")
                .wrap(actix_web::middleware::Logger::default())
                .app_data(state.clone())
                .service(index)
                .configure(dates_service),
        )
        .service(landing)
        .service(search_verification);
    };
    Ok(config.into())
}
