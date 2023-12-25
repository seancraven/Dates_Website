use super::super::domain::postgres_repository::PgUser;
use anyhow::anyhow;
use sqlx::postgres::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;
use tracing::error;

impl From<PgUser> for UnauthorizedUser {
    fn from(u: PgUser) -> Self {
        Self {
            username: u.username,
            email: u.email,
        }
    }
}
#[derive(Debug, Clone)]
pub struct UserPair<'a, 'b>(&'a str, &'b str);
impl<'a, 'b> UserPair<'a, 'b> {
    fn get_partner(&self, username: &str) -> anyhow::Result<&str> {
        if username == self.0 {
            Ok(self.1)
        } else if username == self.1 {
            Ok(self.0)
        } else {
            Err(anyhow!("User isn't in this pair"))
        }
    }
}
#[derive(FromRow, Debug, Clone)]
pub struct UnauthorizedUser {
    username: String,
    email: String,
}
impl UnauthorizedUser {
    fn new(name: String, username: String, email: String) -> Self {
        Self { username, email }
    }
    async fn add_to_db(self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO users (username, email) VALUES ($1, $2)"#,
            self.username,
            self.email
        )
        .execute(pool)
        .await?;
        Ok(())
    }
    async fn from_db(pool: &PgPool, username: &str) -> Option<Self> {
        match sqlx::query_as!(PgUser, r#"SELECT * FROM users WHERE username=$1"#, username)
            .fetch_one(pool)
            .await
        {
            Ok(u) => Some(u.into()),
            Err(e) => {
                error!("Database Query error: {}", e);
                None
            }
        }
    }
}
pub struct AuthorizedUser {
    username: String,
    email: String,
    user_group: i32,
}
