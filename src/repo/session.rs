use crate::{crypto::token::hash_token, error::Result};
use chrono::{DateTime, TimeDelta, Utc};
use rand::{TryRngCore, rngs::OsRng};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_session(
    user_id: &Uuid,
    pool: &PgPool,
) -> Result<(Vec<u8>, chrono::DateTime<Utc>)> {
    // generate token
    let mut token = [0u8; 32];
    OsRng.try_fill_bytes(&mut token)?;
    let hashed_session_token = hash_token(&token);
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
    Ok((token.to_vec(), session.expires_at))
}

pub async fn find_session_by_token(session_token: &[u8], pool: &PgPool) -> Result<Option<Session>> {
    let hashed_session_token = hash_token(session_token);

    let session = sqlx::query_as::<_, Session>(
        r#"
            SELECT id, user_id, token_hash, created_at, last_seen_at, expires_at, revoked_at
            FROM sessions
            WHERE token_hash = $1
            "#,
    )
    .bind(&hashed_session_token)
    .fetch_optional(pool)
    .await?;

    let now = Utc::now();

    if let Some(sess) = session {
        if sess.expires_at < now || sess.revoked_at.is_some() {
            return Ok(None);
        }

        if sess.last_seen_at < now - TimeDelta::minutes(15) {
            sqlx::query(
                r#"
                        UPDATE sessions
                        SET last_seen_at = $1, expires_at = $2
                        WHERE token_hash = $3
                        "#,
            )
            .bind(now)
            .bind(now + TimeDelta::days(5))
            .bind(&hashed_session_token)
            .execute(pool)
            .await?;
        }
        Ok(Some(sess))
    } else {
        Ok(None)
    }
}

pub async fn revoke_session(session_id: &Uuid, pool: &PgPool) -> Result<()> {
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
