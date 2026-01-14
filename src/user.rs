use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::error::{Error, Result};

/// Dummy PHC hash used to equalize timing when the user does not exist.
const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$pmcRQCyDK/zHE03SpG7d2A$/204Eb0JCZ72yrZ+CUFRCv0X91nkLyNix8lQxRFyD5g";

// Private DB model (never exposed outside this module)
#[derive(sqlx::FromRow)]
struct User {
    id: Uuid,
    username: String,
    pass_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    async fn find_by_username(username: &str, pool: &PgPool) -> Result<Option<Self>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, pass_hash, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }
}

#[derive(Serialize)]
pub struct UserPublic {
    id: Uuid,
    username: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[instrument(skip(password, pool))]
pub async fn create_user(
    username: &str,
    password: &str,
    pool: &PgPool,
    argon_params: Params,
) -> Result<UserPublic> {
    let phc = hash_password_async(password.to_string(), argon_params).await?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, pass_hash)
        VALUES ($1, $2)
        RETURNING id, username, pass_hash, created_at, updated_at
        "#,
    )
    .bind(username)
    .bind(phc)
    .fetch_one(pool)
    .await
    .map_err(map_unique_violation)?;

    Ok(UserPublic::from(user))
}

#[instrument(skip(password, pool))]
pub async fn authenticate_user(
    username: &str,
    password: &str,
    pool: &PgPool,
) -> Result<UserPublic> {
    let user_opt = User::find_by_username(username, pool).await?;

    // Always run verification to equalize timing
    let phc: String = match &user_opt {
        Some(user) => user.pass_hash.clone(),
        None => DUMMY_HASH.to_string(),
    };

    let verify_result = verify_password_async(phc, password.as_bytes().to_vec()).await;

    let Some(user) = user_opt else {
        return Err(Error::WrongCredentials);
    };

    verify_result.map_err(|_| Error::WrongCredentials)?;

    Ok(UserPublic::from(user))
}

fn map_unique_violation(e: sqlx::Error) -> Error {
    if let sqlx::Error::Database(db_err) = &e {
        // Postgres unique_violation is SQLSTATE 23505
        if db_err.code().as_deref() == Some("23505") {
            return Error::UserAlreadyExists;
        }
    }
    Error::Sqlx(e)
}

fn hash_password(password: &str, argon_params: Params) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon_params,
    );
    let hashed_password = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hashed_password.to_string())
}

pub async fn hash_password_async(password: String, argon_params: Params) -> Result<String> {
    let phc = tokio::task::spawn_blocking(move || hash_password(&password, argon_params)).await??;
    Ok(phc)
}

fn verify_password(phc: &str, password: &[u8]) -> Result<()> {
    let parsed = PasswordHash::new(phc)?;
    // Params are not required for the verification because they are extracted from the phc
    Argon2::default().verify_password(password, &parsed)?;
    Ok(())
}

async fn verify_password_async(phc: String, password: Vec<u8>) -> Result<()> {
    tokio::task::spawn_blocking(move || verify_password(&phc, &password)).await??;
    Ok(())
}

pub async fn health_check(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        SELECT 1
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}
