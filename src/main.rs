use actix_web::web::ServiceConfig;
use actix_web::web::{self, Data};
use dates::backend::in_memory::VecRepo;
use dates::backend::postgres::PgRepo;
use dates::domain::repository::AppState;
use dates::routes::dates_service::{add_new_date, dates_service};
use dates::routes::landing::{create_user, dummy_login, landing, search_verification};
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::PgPool;

// postgres://postgres:postgres@localhost:17972/postgres
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:assword@localhost:5432/postgres"
    )]
    pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    sqlx::migrate!().run(&pool).await.unwrap();
    // let state = web::Data::new(AppState::new(Box::new(VecRepo::new())));
    // let state = AppState::new_in_web_data(Box::new(PgRepo { pool }));
    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(AppState::new_in_web_data(Box::new(PgRepo { pool })))
            .service(add_new_date)
            .service(landing)
            .service(dummy_login)
            .service(create_user)
            .service(search_verification)
            .service(
                web::scope("/dates")
                    .wrap(actix_web::middleware::Logger::default())
                    .configure(dates_service),
            );
    };
    Ok(config.into())
}
