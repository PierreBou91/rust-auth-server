// Error handling
// from Jeremy Chone excellent video on error handling
// https://www.youtube.com/watch?v=j-VQCYP7wyw
pub type Result<T> = std::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

use std::time::Duration;

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, Pool, Postgres, postgres::PgPoolOptions};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct UserProvidedInfo {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: Uuid,
    username: String,
    pass_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    async fn create(username: String, pass_hash: String, pool: Pool<Postgres>) -> Result<Self> {
        let user = sqlx::query_as::<_, User>(
            "
        INSERT INTO
        users (username,pass_hash)
        VALUES ($1,$2)
        RETURNING *; 
            ",
        )
        .bind(username)
        .bind(pass_hash)
        .fetch_one(&pool)
        .await?;

        // TODO: Error handling - currently returning sqlx error for simplicity, but we should wrap with our own error type or other error handling
        Ok(user)
    }

    async fn find_by_username(username: String, pool: Pool<Postgres>) -> Result<Option<Self>> {
        let user = sqlx::query_as::<_, User>(
            "
        SELECT *
        FROM users
        WHERE username = $1;
        ",
        )
        .bind(username)
        .fetch_optional(&pool)
        .await?;

        // TODO: Error handling - currently returning sqlx error for simplicity, but we should wrap with our own error type or other error handling
        Ok(user)
    }

    async fn verify_password(&self, password: &[u8]) -> Result<()> {
        let parsed = PasswordHash::new(&self.pass_hash)
            .map_err(|e| format!("Invalid password hash: {:}", e))?;
        match Argon2::default().verify_password(password, &parsed).is_ok() {
            true => Ok(()),
            false => Err("Invalid password".into()),
        }
    }
}

// #[derive(Serialize, Deserialize, sqlx::FromRow)]
// struct Session {
//     id: Uuid,
//     user_id: Uuid,
//     session_token: Vec<u8>,
//     pass_hash: String,
//     created_at: DateTime<Utc>,
//     updated_at: DateTime<Utc>,
// }

#[tokio::main]
async fn main() -> Result<()> {
    // RUST_LOG=debug cargo run
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await?;

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<UserProvidedInfo>,
) -> axum::response::Result<Json<Value>> {
    let phc = hash_password(&payload.password).unwrap();

    // TODO: Sanitization of payload
    Ok(Json(json!(
        User::create(payload.username, phc, pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    )))
}

async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<UserProvidedInfo>,
) -> axum::response::Result<Json<Value>> {
    let user = User::find_by_username(payload.username, pool.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(user) = &user {
        user.verify_password(payload.password.as_bytes())
            .await
            .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    }

    match user {
        Some(user) => Ok(Json(json!(user))),
        None => Err("Nobody to be found".into()),
    }
}

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // TODO: Tune params
    let hashed_password = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("hashed pasword error: {:}", e))?;
    Ok(hashed_password.to_string())
}
