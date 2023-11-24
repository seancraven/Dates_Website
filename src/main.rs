mod routes;

use actix_web::{web, App, HttpServer};
use routes::{index, update_date, Date, DateRepo};
use std::sync::Mutex;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(Mutex::new(DateRepo {
                dates: vec![Date::new("1".into()), Date::new("2".into())],
            })))
            .service(index)
            .service(update_date)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await?;
    Ok(())
}
