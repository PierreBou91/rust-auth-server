use chrono::{DateTime, TimeDelta, Utc};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
// CREATE TABLE sessions (
//     id UUID PRIMARY KEY DEFAULT uuidv7(),
//     user_id UUID REFERENCES users(id) ON DELETE CASCADE,
//     token_hash BYTEA NOT NULL UNIQUE,
//     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
//     last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
//     expires_at TIMESTAMPTZ NOT NULL,
//     revoked_at TIMESTAMPTZ
// )
pub async fn create_session(user_id: &Uuid, pool: &PgPool) -> Result<Vec<u8>> {
    // generate token
    let mut token = [0u8; 32];
    OsRng.try_fill_bytes(&mut token)?;
    let mut hasher = Sha256::new();
    hasher.update(token);
    let tmp = hasher.finalize();
    let hashed_session_token = tmp.as_slice();
    let now = Utc::now();
    let days = TimeDelta::days(5);
    let then = now + days;
    let session = sqlx::query_as::<_, Session>(
        r#"
            INSERT INTO sessions (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, token_hash, created_at, last_seen_at, expires_at, revoked_at
            "#,
    )
    .bind(user_id)
    .bind(hashed_session_token)
    .bind(then)
    .fetch_one(pool)
    .await?;
    // return token to caller
    Ok(session.token_hash)
}

pub async fn find_session_by_token(
    session_token: &Vec<u8>,
    pool: &PgPool,
) -> Result<Option<Session>> {
    let session = sqlx::query_as::<_, Session>(
        r#"
            SELECT id, user_id, token_hash, created_at, last_seen_at, expires_at, revoked_at
            FROM sessions
            WHERE token_hash == $1
            "#,
    )
    .bind(session_token)
    .fetch_one(pool)
    .await?;
    Ok(Some(session))
}

pub async fn revoke_session(session_id: &str, pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
            UPDATE sessions
            SET revoked_at = NOW()
            WHERE id = $1
            "#,
    )
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}
