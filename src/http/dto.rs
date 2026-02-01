use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain;

#[derive(Serialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<domain::User> for PublicUser {
    fn from(value: domain::User) -> Self {
        PublicUser {
            id: value.id,
            username: value.username,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Deserialize)]
pub struct RegisterUser {
    pub username: String,
    pub password: String,
}
