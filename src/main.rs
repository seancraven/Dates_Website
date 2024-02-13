use actix_web::web::ServiceConfig;
use anyhow::Context;
use date_rs::routes::landing::MainService;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::{Pool, Postgres};
// TODO:
//  - [x] Make Login Stuff Nicer.
//  - [x] Add email backend for Authorization and Inviting.
//  - [x] Change structure of backend.
//  - [x] Integrate email backend.
//  - [ ] Figure out how to manage not having a user in the database.
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:assword@localhost:5432/postgres"
    )]
    conn_str: String,
    #[shuttle_secrets::Secrets] secrets: shuttle_secrets::SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let email_client = date_rs::email::EmailClient::new(
        secrets.get("postmark_api_token").unwrap(),
        secrets.get("url").unwrap(),
        secrets.get("from_email").unwrap(),
    );
    let pool = Pool::<Postgres>::connect(&conn_str)
        .await
        .context("Db connection failed")?;
    sqlx::migrate!().run(&pool).await.unwrap();
    let config = move |cfg: &mut ServiceConfig| {
        MainService::new(pool, email_client.clone()).service_configuration(cfg)
    };
    Ok(config.into())
}
