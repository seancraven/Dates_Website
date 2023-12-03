mod domain;
mod routes;
mod templating;

use actix_web::web;
use actix_web::web::ServiceConfig;
use domain::postgres_repository::PgRepo;
use domain::repository::AppState;
use routes::dates::dates_service;
use routes::index::index;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    // let db_url = secret_store.get("DATABASE_URL").unwrap();
    // std::env::set_var("DATABASE_URL", db_url);
    sqlx::migrate!().run(&pool).await.unwrap();
    let state = AppState::new(Box::new(PgRepo { pool }));
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
