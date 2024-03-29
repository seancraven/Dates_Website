use actix_web::web::ServiceConfig;
use anyhow::Context;
use date_rs::routes::landing::MainService;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::{Pool, Postgres};
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:assword@localhost:5432/postgres"
    )]
    conn_str: String,
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let email_client = date_rs::email::EmailClient::new(
        secrets
            .get("postmark_api_token")
            .expect("Set postmark_api_token"),
        secrets.get("url").expect("Set url"),
        secrets.get("from_email").expect("Set from_email"),
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
