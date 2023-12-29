pub mod user;

use anyhow::{anyhow, Context};
use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use secrecy::{ExposeSecret, Secret};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Password Hashes don't match.")]
    AuthenticationError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub async fn verify_password_hash<'a>(
    password: Secret<String>,
    password_hash: Secret<String>,
) -> Result<(), PasswordError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .verify_password(
                password.expose_secret().as_bytes(),
                &PasswordHash::new(password_hash.expose_secret())
                    .context("Failed to parse password.")?,
            )
            .context("Invalid password.")
            .map_err(PasswordError::AuthenticationError)
    })
    .await
    .context("Join error")?
}

pub async fn compute_password_hash(password: Secret<String>) -> anyhow::Result<Secret<String>> {
    let string_hash = tokio::task::spawn_blocking(move || {
        Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(1500, 2, 1, None).unwrap(),
        )
        .hash_password(
            password.expose_secret().as_bytes(),
            &SaltString::generate(rand::thread_rng()),
        )
        .unwrap()
        .to_string()
    })
    .await
    .context("Join error.")?;
    Ok(Secret::new(string_hash))
}
#[cfg(test)]
mod tests {
    use secrecy::Secret;

    use super::{compute_password_hash, verify_password_hash};

    #[tokio::test]
    async fn test_password_accept() -> anyhow::Result<()> {
        let dummy_pword = Secret::new(String::from("Sexy As Funk"));
        let hash_str = compute_password_hash(dummy_pword.clone()).await?;
        verify_password_hash(dummy_pword, hash_str).await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_password_fail() -> anyhow::Result<()> {
        let dummy_pword = Secret::new(String::from("Sexy As Funk"));
        let hash_str = compute_password_hash(dummy_pword.clone()).await?;
        assert!(
            verify_password_hash(Secret::new(String::from("Not Sexy As Funk")), hash_str)
                .await
                .is_err()
        );
        Ok(())
    }
}
