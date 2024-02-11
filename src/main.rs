use actix_web::web::ServiceConfig;
use anyhow::Context;
use dates::routes::landing::MainService;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::{Pool, Postgres};
// TODO:
//  - [ ] Make Login Stuff Nicer.
//  - [ ] Add email backend for Authorization and Inviting.
//  - [ ] Test
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:assword@localhost:5432/postgres"
    )]
    conn_str: String,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let pool = Pool::<Postgres>::connect(&conn_str)
        .await
        .context("Db connection failed")?;
    sqlx::migrate!().run(&pool).await.unwrap();
    let config = move |cfg: &mut ServiceConfig| MainService::new(pool).service_configuration(cfg);
    Ok(config.into())
}
