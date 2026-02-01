use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
