use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Error, Result};

// private struct to avoid exposing the password hash to the web server.
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
        let optional_user = sqlx::query_as::<_, User>(
            r#"
            SELECT *
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(optional_user)
    }

    async fn verify_password(&self, password: &[u8]) -> Result<()> {
        let parsed = PasswordHash::new(&self.pass_hash)?;
        Argon2::default().verify_password(password, &parsed)?;
        Ok(())
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

pub async fn create_user(username: &str, password: &str, pool: &PgPool) -> Result<UserPublic> {
    let phc = hash_password(password)?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, pass_hash)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(username)
    .bind(phc)
    .fetch_one(pool)
    .await
    .map_err(map_unique_violation)?;

    Ok(UserPublic::from(user))
}

pub async fn authenticate_user(
    username: &str,
    password: &str,
    pool: &PgPool,
) -> Result<UserPublic> {
    let user = User::find_by_username(username, pool)
        .await?
        .ok_or(Error::WrongCredentials)?;

    user.verify_password(password.as_bytes())
        .await
        .map_err(|_| Error::WrongCredentials)?;

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

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // TODO: Tune params
    let hashed_password = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hashed_password.to_string())
}
