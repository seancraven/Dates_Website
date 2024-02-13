//! User Createion Flow:
//! 1) User creates an account.
//! UnRegisteredUser -> new_usier_id.
//! 2) User Activates their account.
//! uuid -> NoGroupUser.
//! 3) User joins a Date group.
//! NoGroupUser -> GroupUser.
//!
//! User Login Flow:
//! 1) User logs in:
//! UnAuthorizedUser -> AuthorizedUser

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
    async fn add_user_to_new_group(&self, user: NoGroupUser) -> anyhow::Result<GroupUser> {
        self.add_user_to_group(user, self.create_group().await?)
            .await
    }
    /// Create a user and add them to an existing group.
    ///
    /// * `user`: New user without a group.
    /// * `email`: email of a member of the target group.
    async fn add_user_to_group_by_email(
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
    /// Validate a user, take a user with a password and return their grouped status.
    ///
    /// * `user`:
    async fn validate_user(
        &self,
        user: &UnAuthorizedUser,
    ) -> Result<AuthorizedUser, UserValidationError>;
    /// Add a user to the database.
    ///
    /// * `user`: A user that doesn't exist on the system.
    async fn register_user(&self, user: UnRegisteredUser) -> Result<Uuid, UserValidationError>;
    /// Activates a user account by id.
    ///
    /// * `user`: A user whoes data is in the database.
    async fn activate_user(&self, user_id: &Uuid) -> Result<NoGroupUser, UserValidationError>;
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
    async fn get_user(&self, user_id: &Uuid)
        -> anyhow::Result<AuthorizedUser, UserValidationError>;
}

#[derive(Error, Debug)]
pub enum UserValidationError {
    #[error("Incorect Password.")]
    PasswordError(#[source] anyhow::Error),
    #[error("Unregisterd User.")]
    RegistrationError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
#[derive(Deserialize, Clone)]
/// Unqueryable user, that provides temporary storage.
/// Once added to the system a user gains an id.
///
/// * `email`:
/// * `password`:
pub struct UnRegisteredUser {
    pub email: String,
    pub password: Secret<String>,
}
impl UnRegisteredUser {
    pub fn new(email: impl Into<String>, password: impl Into<String>) -> Self {
        UnRegisteredUser {
            email: email.into(),
            password: Secret::new(password.into()),
        }
    }
}
/// Models a user that should exist, but isn't logged in.
///
#[derive(Deserialize)]
pub struct UnAuthorizedUser {
    pub email: String,
    pub password: Secret<String>,
}
impl UnAuthorizedUser {
    pub fn new(email: impl Into<String>, password: impl Into<String>) -> Self {
        UnAuthorizedUser {
            email: email.into(),
            password: Secret::new(password.into()),
        }
    }
}

#[derive(Deserialize)]
pub enum AuthorizedUser {
    NoGroupUser(NoGroupUser),
    GroupUser(GroupUser),
}

#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct NoGroupUser {
    pub id: Uuid,
    pub email: String,
}
impl NoGroupUser {
    pub fn join_group(self, group: i32) -> GroupUser {
        GroupUser {
            id: self.id,
            email: self.email,
            user_group: group,
        }
    }
    pub fn new(id: Uuid, email: impl Into<String>) -> Self {
        NoGroupUser {
            id,
            email: email.into(),
        }
    }
}
#[derive(Deserialize, Clone, Debug, Serialize)]
#[allow(clippy::module_name_repetitions)]
pub struct GroupUser {
    pub id: Uuid,
    pub email: String,
    pub user_group: i32,
}
impl GroupUser {
    pub fn new(id: Uuid, email: impl Into<String>, user_group: i32) -> Self {
        GroupUser {
            id,
            email: email.into(),
            user_group,
        }
    }
}
