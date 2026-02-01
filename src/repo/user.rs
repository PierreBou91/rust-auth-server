use crate::{
    crypto::password::hash_password_async,
    domain,
    error::{Error, Result},
};
use argon2::Params;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

async fn fetch_user_by_username(username: &str, pool: &PgPool) -> Result<Option<UserRecord>> {
    Ok(sqlx::query_as::<_, UserRecord>(
        r#"
            SELECT id, username, pass_hash, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?)
}

#[instrument(skip(password, pool))]
pub async fn create_user(
    username: &str,
    password: &str,
    pool: &PgPool,
    argon_params: Params,
) -> Result<domain::User> {
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

#[instrument(skip(pool))]
pub async fn find_user_by_id(id: &Uuid, pool: &PgPool) -> Result<UserPublic> {
    let user_opt = sqlx::query_as::<_, User>(
        r#"
            SELECT id, username, pass_hash, created_at, updated_at
            FROM users
            WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match user_opt {
        Some(user) => Ok(UserPublic::from(user)),
        None => Err(Error::Unauthenticated),
    }
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
