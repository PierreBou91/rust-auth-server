use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod pg_session;
pub mod pg_user;

#[derive(sqlx::FromRow)]
pub struct UserRecord {
    pub id: Uuid,
    pub username: String,
    pub pass_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct SessionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn insert_user(&self, username: &str, hashed_password: &str) -> Result<UserRecord>;
    async fn select_by_username(&self, username: &str) -> Result<Option<UserRecord>>; // change &str with type
}
#[async_trait]
pub trait SessionRepo: Send + Sync {
    async fn select_by_token(&self, token: &str) -> Result<SessionRecord>;
    async fn insert_session(&self, session: SessionRecord) -> Result<SessionRecord>;
}
