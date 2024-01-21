use anyhow::{anyhow, Context};
use argon2::PasswordHash;
use chrono::Local;
use log::error;
use secrecy::{ExposeSecret, Secret};
use shuttle_runtime::async_trait;
use sqlx::{
    types::chrono::{DateTime, Utc},
    types::Uuid,
    FromRow, PgPool,
};

use crate::{
    auth::user::{GroupUser, UserValidationError},
    domain::repository::InsertDateError,
};
use crate::{
    auth::user::{NoGroupUser, UserRepository},
    domain::repository::DateRepository,
};
use crate::{auth::verify_password_hash, domain::repository::Repository};
use crate::{
    auth::{
        compute_password_hash,
        user::{AuthorizedUser, UnauthorizedUser},
    },
    domain::dates::{Date, Description, Status},
};
// Databse structures.
#[derive(FromRow, Debug, Clone)]
struct PgDate {
    id: String,
    name: String,
    count_: i32,
    #[sqlx(default)]
    day: Option<DateTime<Utc>>,
    #[sqlx(default)]
    description: Option<String>,
    status: i32,
    #[allow(dead_code)]
    user_group: i32,
}
impl TryInto<Date> for PgDate {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Date, Self::Error> {
        Ok(Date {
            id: uuid::Uuid::parse_str(&self.id)?,
            name: self.name,
            count: self.count_,
            description: Description::new(
                self.description.unwrap_or("".into()),
                match self.status {
                    0 => Status::Suggested,
                    1 => Status::Approved,
                    2 => Status::Rejected,
                    _ => return Err(anyhow::anyhow!("Invalid status")),
                },
                self.day.map(|d| d.with_timezone(&Local)),
            ),
        })
    }
}

/// Postgres Repository
pub struct PgRepo {
    pub pool: PgPool,
}
impl PgRepo {
    async fn get_user_group(&self, user_id: &Uuid) -> anyhow::Result<Option<i32>> {
        let user = sqlx::query!(r#"SELECT user_group FROM users WHERE user_id=$1"#, user_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(user.user_group)
    }
}
#[async_trait]
impl Repository for PgRepo {}
#[async_trait]
impl DateRepository for PgRepo {
    async fn check_user_has_access(&self, user_id: &Uuid) -> bool {
        match sqlx::query!(r#"SELECT user_group FROM users WHERE user_id=$1"#, user_id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(_) => true,
            Err(e) => {
                error!("Database Query error: {}", e);
                false
            }
        }
    }
    async fn add(&self, date: Date, user_id: Uuid) -> Result<(), InsertDateError> {
        let Some(group) = self
            .get_user_group(&user_id)
            .await
            .map_err(|_| InsertDateError::QueryError)?
        else {
            return Err(InsertDateError::GroupMembershipError);
        };
        sqlx::query!(
            r#"INSERT INTO dates (id, name, count_ , day , status,  description, user_group ) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            date.id,
            date.name.clone(),
            date.count,
            date.description.day,
            date.description.status as i32,
            date.description.text,
            group
        )
        .execute(&self.pool)
        .await.map_err(|_| InsertDateError::QueryError)?;
        Ok(())
    }
    async fn get<'a, 'ui, 'st>(&'a self, date_id: &'ui Uuid, user_id: &'st Uuid) -> Option<Date> {
        let group = self.get_user_group(user_id).await.unwrap().unwrap();
        match sqlx::query_as!(
            PgDate,
            r#"SELECT * FROM dates WHERE id=$1 and user_group=$2 "#,
            date_id,
            group
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(d) => match d.try_into() {
                Ok(date) => Some(date),
                Err(e) => {
                    error!("Query conversion error: {} on converting Uuid", e);
                    None
                }
            },
            Err(e) => {
                error!("Database Query error: {}", e);
                None
            }
        }
    }
    async fn remove<'a, 'ui, 'st>(
        &'a self,
        date_id: &'ui Uuid,
        user_id: &'st Uuid,
    ) -> anyhow::Result<()> {
        let group = self.get_user_group(user_id).await.unwrap().unwrap();
        sqlx::query!(
            r#"DELETE FROM dates WHERE id=$1 and user_group=$2"#,
            date_id,
            group,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn get_all(&self, user_id: &Uuid) -> Vec<Date> {
        let group = self.get_user_group(user_id).await.unwrap().unwrap();
        match sqlx::query_as!(PgDate, r#"SELECT * FROM dates where user_group=$1"#, group)
            .fetch_all(&self.pool)
            .await
        {
            Ok(d) => {
                let mut v = d
                    .into_iter()
                    .filter_map(|d| d.try_into().ok())
                    .collect::<Vec<Date>>();
                v.sort_by(|a, b| b.count.cmp(&a.count));
                v
            }

            Err(e) => {
                error!("Database Query error: {}", e);
                vec![]
            }
        }
    }
    async fn decrement_date_count<'a, 'ui, 'st>(
        &'a self,
        date_id: &'ui Uuid,
        user_id: &'st Uuid,
    ) -> anyhow::Result<()> {
        let group = self.get_user_group(user_id).await.unwrap().unwrap();
        sqlx::query!(
            r#"UPDATE dates SET count_=count_-1 WHERE id = $1 and count_ > 0 and user_group=$2"#,
            date_id,
            group
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn increment_date_count<'a, 'ui, 'st>(
        &'a self,
        date_id: &'ui Uuid,
        user_id: &'st Uuid,
    ) -> anyhow::Result<()> {
        let group = self.get_user_group(user_id).await.unwrap().unwrap();
        sqlx::query!(
            r#"UPDATE dates SET count_=count_+1 WHERE id = $1 and user_group=$2"#,
            date_id,
            group
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, date: Date, user_id: &Uuid) -> anyhow::Result<()> {
        let group = self.get_user_group(user_id).await.unwrap().unwrap();
        sqlx::query!(
            r#"UPDATE dates SET count_=$3, name=$4, day=$5, status=$6,  description=$7 WHERE id = $1 and user_group=$2"#,
            date.id,
            group,
            date.count,
            date.name,
            date.description.day,
            date.description.status as i32,
            date.description.text,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
#[derive(FromRow, Debug, Clone)]
pub struct PgUser {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub user_group: Option<i32>,
}
impl PgUser {
    fn into_user(self) -> AuthorizedUser {
        match self.user_group {
            Some(g) => AuthorizedUser::GroupUser(GroupUser {
                id: self.user_id,
                username: self.username,
                email: self.email,
                user_group: g,
            }),
            None => AuthorizedUser::NoGroupUser(NoGroupUser {
                id: self.user_id,
                username: self.username,
                email: self.email,
            }),
        }
    }
}
#[async_trait]
impl UserRepository for PgRepo {
    async fn add_user_to_group(&self, user: NoGroupUser, group: i32) -> anyhow::Result<GroupUser> {
        let a_user = user.join_group(group);
        sqlx::query!(
            r#"INSERT INTO users (user_id, username, email, user_group) VALUES ($1, $2,$3, $4);"#,
            a_user.id,
            &a_user.username,
            &a_user.email,
            group,
        )
        .execute(&self.pool)
        .await
        .context("Query failed.")?;
        Ok(a_user)
    }
    async fn create_group(&self) -> anyhow::Result<i32> {
        sqlx::query_scalar!(r#"INSERT INTO user_groups DEFAULT VALUES RETURNING id;"#)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!(e))
    }
    async fn get_group_by_email(&self, email: &str) -> anyhow::Result<i32> {
        sqlx::query_scalar!(r#"SELECT (user_group) FROM users WHERE email=$1"#, email)
            .fetch_one(&self.pool)
            .await?
            .ok_or(anyhow!("No group found from email."))
    }
    async fn validate_user(
        &self,
        user: &UnauthorizedUser,
    ) -> Result<AuthorizedUser, crate::auth::user::UserValidationError> {
        let expectec_user = sqlx::query_as!(
            PgUser,
            r#"SELECT * FROM users WHERE username=$1 and email=$2;"#,
            user.username,
            user.email,
        )
        .fetch_one(&self.pool)
        .await
        .context("Query failed.")?;
        let password_hash = PasswordHash::new(
            expectec_user
                .password_hash
                .as_ref()
                .ok_or(UserValidationError::PasswordError(anyhow!("No Password.")))?,
        )
        .context("Parsing hash failed")?;
        verify_password_hash(
            user.password.clone(),
            Secret::new(password_hash.to_string()),
        )
        .await
        .map_err(|e| UserValidationError::PasswordError(e.into()))?;
        Ok(expectec_user.into_user())
    }

    async fn remove_user(&self, user_id: &Uuid) -> anyhow::Result<()> {
        todo!();
    }
    async fn create_authorized_user(&self, user: UnauthorizedUser) -> anyhow::Result<NoGroupUser> {
        let password_hash = compute_password_hash(user.password).await?;
        let new_authorized_user = NoGroupUser {
            id: Uuid::new_v4(),
            username: user.username,
            email: user.email,
        };
        sqlx::query!(
            r#"INSERT INTO users (user_id, username, email, password_hash) VALUES ($1, $2, $3, $4);"#,
            new_authorized_user.id,
            new_authorized_user.username,
            new_authorized_user.email,
            password_hash.expose_secret(),
        ).execute(&self.pool).await.context("Query error")?;
        Ok(new_authorized_user)
    }
    async fn change_user_password(
        &self,
        user: AuthorizedUser,
        new_password: secrecy::Secret<String>,
    ) -> anyhow::Result<AuthorizedUser> {
        todo!();
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use sqlx::PgPool;
    use tracing;
    fn tracing_once() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    }

    #[tokio::test]
    async fn test_repo() -> anyhow::Result<()> {
        tracing_once();
        let pool = PgPool::connect("postgres://postgres:assword@localhost:5432/postgres")
            .await
            .unwrap();
        let repo = PgRepo { pool };
        let no_g_use = NoGroupUser {
            id: Uuid::new_v4(),
            username: String::from("Test"),
            email: String::from("test"),
        };
        let g = repo.create_user_and_group(no_g_use).await?;
        repo.add(Date::new("Test"), g.id).await?;
        Ok(())
    }
}
