use super::super::domain::postgres_repository::PgUser;
use anyhow::anyhow;
use serde::Deserialize;
use sqlx::postgres::PgPool;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::FromRow;
use tracing::error;
use uuid::Uuid;

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
#[derive(FromRow, Debug, Clone, Deserialize)]
pub struct UnauthorizedUser {
    username: String,
    email: String,
}
impl UnauthorizedUser {
    fn new(name: String, username: String, email: String) -> Self {
        Self { username, email }
    }
    pub async fn create_user_and_group(self, pool: &PgPool) -> anyhow::Result<AuthorizedUser> {
        let group = create_group(pool).await?;
        self.join_group(pool, group).await
    }
    pub async fn join_group(
        self,
        pool: &PgPool,
        user_group: i32,
    ) -> anyhow::Result<AuthorizedUser> {
        let id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO users (user_id, username, email, user_group) VALUES ($1, $2,$3, $4);"#,
            id,
            &self.username,
            &self.email,
            user_group,
        )
        .execute(pool)
        .await?;
        Ok(AuthorizedUser {
            id,
            username: self.username,
            email: self.email,
            user_group,
        })
    }
    /// Creates a new AuthorizedUser as part of an existing group through an email
    ///
    /// * `pool`: PgPool
    /// * `email`: Email of member that is in group.
    pub async fn create_and_join_by_email(
        self,
        pool: &PgPool,
        email: &str,
    ) -> anyhow::Result<AuthorizedUser> {
        let group = sqlx::query_scalar!(r#"SELECT (user_group) FROM users WHERE email=$1"#, email)
            .fetch_one(pool)
            .await?
            .ok_or(anyhow!("No group found from email."))?;
        self.join_group(pool, group).await
    }
}
pub struct AuthorizedUser {
    id: Uuid,
    username: String,
    email: String,
    user_group: i32,
}
async fn create_group(pool: &PgPool) -> anyhow::Result<i32> {
    sqlx::query_scalar!(r#"INSERT INTO user_groups DEFAULT VALUES RETURNING id;"#)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!(e))
}
