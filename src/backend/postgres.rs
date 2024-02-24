use anyhow::{anyhow, Context};
use argon2::PasswordHash;
use chrono::Local;
use secrecy::{ExposeSecret, Secret};
use shuttle_runtime::async_trait;
use sqlx::{
    types::chrono::{DateTime, Utc},
    types::Uuid,
    FromRow, PgPool,
};
use tracing::error;

use crate::{
    auth::user::UnAuthorizedUser,
    domain::repository::{DateRepository, InsertDateError, Repository},
};
use crate::{
    auth::{
        compute_password_hash,
        user::{
            AuthorizedUser, GroupUser, NoGroupUser, UnRegisteredUser, UserRepository,
            UserValidationError,
        },
        verify_password_hash,
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
    pub email: String,
    pub password_hash: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub user_group: Option<i32>,
    pub auth: bool,
}
impl TryInto<AuthorizedUser> for PgUser {
    type Error = UserValidationError;
    fn try_into(self) -> Result<AuthorizedUser, Self::Error> {
        if !self.auth {
            return Err(UserValidationError::RegistrationError(anyhow!(
                "User isn't registerd."
            )));
        }
        Ok(match self.user_group {
            Some(g) => AuthorizedUser::GroupUser(GroupUser::new(self.user_id, self.email, g)),
            None => AuthorizedUser::NoGroupUser(NoGroupUser::new(self.user_id, self.email)),
        })
    }
}
#[async_trait]
impl UserRepository for PgRepo {
    async fn get_user_by_email(
        &self,
        user_email: &str,
    ) -> Result<AuthorizedUser, UserValidationError> {
        let expected_user =
            sqlx::query_as!(PgUser, r#"SELECT * FROM users WHERE email=$1;"#, user_email)
                .fetch_one(&self.pool)
                .await
                .map_err(|_| {
                    UserValidationError::RegistrationError(anyhow!(
                        "No user found with email: {:?}",
                        user_email
                    ))
                })?;
        Ok(expected_user.try_into()?)
    }
    async fn get_user(&self, user_id: &Uuid) -> Result<AuthorizedUser, UserValidationError> {
        let expected_user =
            sqlx::query_as!(PgUser, r#"SELECT * FROM users WHERE user_id=$1;"#, user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|_| {
                    UserValidationError::RegistrationError(anyhow!(
                        "No user found with id: {:?}",
                        user_id
                    ))
                })?;
        Ok(expected_user.try_into()?)
    }
    async fn add_user_to_group(&self, user: NoGroupUser, group: i32) -> anyhow::Result<GroupUser> {
        let a_user = user.join_group(group);
        sqlx::query!(
            r#"UPDATE users SET user_group=$3 WHERE user_id=$1 and email=$2;"#,
            a_user.user_id,
            &a_user.email,
            group,
        )
        .execute(&self.pool)
        .await
        .context("Query failed.")?;
        Ok(a_user)
    }
    async fn create_group(&self) -> anyhow::Result<i32> {
        Ok(
            sqlx::query_scalar!(r#"INSERT INTO user_groups DEFAULT VALUES RETURNING id;"#)
                .fetch_one(&self.pool)
                .await
                .context("Query error on creating a group")?,
        )
    }
    async fn get_group_by_email(&self, email: &str) -> anyhow::Result<i32> {
        sqlx::query_scalar!(r#"SELECT (user_group) FROM users WHERE email=$1"#, email)
            .fetch_one(&self.pool)
            .await?
            .ok_or(anyhow!("No group found from email."))
    }
    async fn validate_user(
        &self,
        user: &UnAuthorizedUser,
    ) -> Result<AuthorizedUser, crate::auth::user::UserValidationError> {
        let expected_user = sqlx::query_as!(
            PgUser,
            r#"SELECT * FROM users WHERE  email=$1;"#,
            user.email,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| {
            UserValidationError::RegistrationError(anyhow!("User :{} doesn't exist.", user.email))
        })?;
        let password_hash = PasswordHash::new(
            expected_user
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
        Ok(expected_user.try_into()?)
    }

    async fn remove_user(&self, user_id: &Uuid) -> anyhow::Result<()> {
        sqlx::query!(r#"DELETE FROM users WHERE user_id=$1"#, user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    async fn register_user(&self, user: UnRegisteredUser) -> Result<Uuid, UserValidationError> {
        let new_id = Uuid::new_v4();
        let password_hash = compute_password_hash(user.password).await?;
        sqlx::query!(
            r#"INSERT INTO users (user_id, email, password_hash) VALUES ($1, $2, $3 );"#,
            new_id,
            user.email,
            password_hash.expose_secret(),
        )
        .execute(&self.pool)
        .await
        .context("Query error")?;
        Ok(new_id)
    }
    async fn change_user_password(
        &self,
        user_id: &Uuid,
        new_password: secrecy::Secret<String>,
    ) -> anyhow::Result<()> {
        let password_hash = compute_password_hash(new_password).await?;
        sqlx::query!(
            r#"UPDATE users SET password_hash=$2 WHERE user_id=$1;"#,
            user_id,
            password_hash.expose_secret(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn activate_user(&self, user_id: &Uuid) -> Result<NoGroupUser, UserValidationError> {
        let record = sqlx::query!(
            r#"UPDATE users SET auth=true WHERE user_id = $1 RETURNING user_id, email;"#,
            user_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UserValidationError::RegistrationError(anyhow!("User doesn't exist")))?;
        Ok(NoGroupUser::new(record.user_id, record.email))
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use sqlx::PgPool;
    async fn setup_repo() -> PgRepo {
        let pool = PgPool::connect("postgres://postgres:assword@localhost:5432/postgres")
            .await
            .unwrap();
        PgRepo { pool }
    }
    #[tokio::test]
    async fn test_create_user_flow() -> anyhow::Result<()> {
        let repo = setup_repo().await;
        let test_user = UnRegisteredUser::new("test_create@unit.com", "assword");
        let id = repo.register_user(test_user).await?;
        repo.remove_user(&id).await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_authorize_user() -> anyhow::Result<()> {
        let repo = setup_repo().await;
        let test_user = UnRegisteredUser::new("test_auth@unit.com", "assword");
        let id = repo.register_user(test_user.clone()).await?;
        let new_u = repo.activate_user(&id).await?;
        assert_eq!(new_u.email, test_user.email);
        repo.remove_user(&id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_user() -> anyhow::Result<()> {
        let repo = setup_repo().await;
        let email = "test_valid@password.com";
        let password = "assword";
        let test_user = UnRegisteredUser::new(email, password);
        let un_auth = UnAuthorizedUser::new(email, password);
        let id = repo.register_user(test_user.clone()).await?;
        repo.activate_user(&id).await?;
        repo.validate_user(&un_auth).await?;
        repo.remove_user(&id).await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_date() -> anyhow::Result<()> {
        let repo = setup_repo().await;
        let test_user = UnRegisteredUser::new("test_date@unit.com", "assword");
        let id = repo.register_user(test_user.clone()).await?;
        let no_g_use = repo.activate_user(&id).await?;
        let g = repo.add_user_to_new_group(no_g_use).await?;
        repo.add(Date::new("Test"), g.user_id).await?;
        repo.remove_user(&id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_password_change() -> anyhow::Result<()> {
        let repo = setup_repo().await;
        let email = "password@unit.com";
        let pword = "assword";
        let password_new = "updated_password";
        let test_user = UnRegisteredUser::new(email, pword);
        let re_login = UnAuthorizedUser::new(email, pword);
        let id = repo.register_user(test_user).await?;
        repo.activate_user(&id)
            .await
            .expect("User activation failed.");
        repo.validate_user(&re_login)
            .await
            .expect("User validation failed.");
        repo.change_user_password(&id, Secret::new(password_new.into()))
            .await
            .expect("Password change failed.");
        let new_u = UnAuthorizedUser::new(email, password_new);
        repo.validate_user(&new_u)
            .await
            .expect("User validation failed.");
        repo.remove_user(&id).await?;
        Ok(())
    }
}
