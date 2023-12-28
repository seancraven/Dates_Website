use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use shuttle_runtime::async_trait;
use sqlx::FromRow;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository {
    async fn create_user_and_group(
        &self,
        user: UnauthorizedUser,
    ) -> anyhow::Result<AuthorizedUser> {
        self.add_user_to_group(user, self.create_group().await?)
            .await
    }
    async fn create_user_and_join_by_email(
        &self,
        user: UnauthorizedUser,
        email: &str,
    ) -> anyhow::Result<AuthorizedUser> {
        self.add_user_to_group(user, self.get_group_by_email(email).await?)
            .await
    }
    async fn add_user_to_group(
        &self,
        user: UnauthorizedUser,
        group: i32,
    ) -> anyhow::Result<AuthorizedUser>;
    async fn create_group(&self) -> anyhow::Result<i32>;
    async fn get_group_by_email(&self, email: &str) -> anyhow::Result<i32>;
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
    pub username: String,
    pub email: String,
}
impl UnauthorizedUser {
    fn new(name: String, username: String, email: String) -> Self {
        Self { username, email }
    }
}
#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct AuthorizedUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub user_group: i32,
}
impl AuthorizedUser {
    pub fn authorize(user: UnauthorizedUser, group: i32) -> AuthorizedUser {
        AuthorizedUser {
            id: Uuid::new_v4(),
            username: user.username,
            email: user.email,
            user_group: group,
        }
    }
}
