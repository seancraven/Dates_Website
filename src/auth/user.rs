use secrecy::Secret;
use serde::{Deserialize, Serialize};
use shuttle_runtime::async_trait;
use sqlx::FromRow;
use thiserror::Error;
use uuid::Uuid;

#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait UserRepository {
    /// Create a user and a group.
    ///
    /// Primary method for making a new user and founding a group.
    ///
    /// * `user`: New user, that doesn't have a target group to join.
    async fn create_user_and_group(&self, user: NoGroupUser) -> anyhow::Result<GroupUser> {
        self.add_user_to_group(user, self.create_group().await?)
            .await
    }
    /// Create a user and add them to an existing group.
    ///
    /// * `user`: New user without a group.
    /// * `email`: email of a member of the target group.
    async fn create_user_and_join_by_email(
        &self,
        user: NoGroupUser,
        email: &str,
    ) -> anyhow::Result<GroupUser> {
        self.add_user_to_group(user, self.get_group_by_email(email).await?)
            .await
    }
    // To implememnt
    /// Add a user to a group via group id.
    async fn add_user_to_group(&self, user: NoGroupUser, group: i32) -> anyhow::Result<GroupUser>;
    /// Create a new group.
    async fn create_group(&self) -> anyhow::Result<i32>;
    /// Find a group by the email of a member.
    ///
    /// * `email`: Group member's email.
    async fn get_group_by_email(&self, email: &str) -> anyhow::Result<i32>;
    /// Validate a user.
    ///
    /// * `user`:
    async fn validate_user(
        &self,
        user: &UnauthorizedUser,
    ) -> Result<AuthorizedUser, UserValidationError>;
    /// Add a user to the repository. With their selected password.
    ///
    /// * `user`: A user without an account.
    async fn create_authorized_user(&self, user: UnauthorizedUser) -> anyhow::Result<NoGroupUser>;
    /// Update an existing authorized users's password.
    async fn change_user_password(
        &self,
        user: AuthorizedUser,
        new_password: Secret<String>,
    ) -> anyhow::Result<AuthorizedUser>;
    async fn remove_user(&self, user_id: &Uuid) -> anyhow::Result<()>;
    /// Get a user from the repository by id.
    ///
    /// * `user_id`: User's id.
    async fn get_user(&self, user_id: &Uuid) -> anyhow::Result<AuthorizedUser>;
}
#[derive(Error, Debug)]
pub enum UserValidationError {
    #[error("Incorect Password.")]
    PasswordError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(Deserialize)]
pub enum AuthorizedUser {
    NoGroupUser(NoGroupUser),
    GroupUser(GroupUser),
}
#[derive(FromRow, Debug, Clone, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct UnauthorizedUser {
    pub email: String,
    pub password: Secret<String>,
}

#[derive(FromRow, Debug, Clone, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct NoGroupUser {
    pub id: Uuid,
    pub email: String,
}
#[derive(Deserialize, Clone, Debug, Serialize)]
#[allow(clippy::module_name_repetitions)]
pub struct GroupUser {
    pub id: Uuid,
    pub email: String,
    pub user_group: i32,
}
impl NoGroupUser {
    pub fn join_group(self, group: i32) -> GroupUser {
        GroupUser {
            id: Uuid::new_v4(),
            email: self.email,
            user_group: group,
        }
    }
}
