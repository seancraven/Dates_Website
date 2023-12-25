use actix_web::web::ServiceConfig;
use sqlx::postgres::PgPool;

pub mod user;

#[derive(Debug)]
pub struct GroupMember {}
