mod routes;

use actix_web::{App, HttpServer};
use routes::index;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind(("127.0.0.1", 8000))?
        .run()
        .await?;
    Ok(())
}
