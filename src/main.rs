use actix_web::web::ServiceConfig;
use dates::routes::landing::MainService;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::PgPool;
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:assword@localhost:5432/postgres"
    )]
    pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    sqlx::migrate!().run(&pool).await.unwrap();
    let config = move |cfg: &mut ServiceConfig| MainService::new(pool).service_configuration(cfg);
    Ok(config.into())
}
