use std::time::Duration;

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{self, PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Error, PgPool, Pool, Postgres, postgres::PgPoolOptions};
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
    async fn create(
        username: String,
        pass_hash: String,
        pool: Pool<Postgres>,
    ) -> Result<Self, Error> {
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
        .await;

        // TODO: Error handling - currently returning sqlx error for simplicity, but we should wrap with our own error type or other error handling
        match user {
            Ok(user) => Ok(user),
            Err(e) => Err(e),
        }
    }

    async fn find_by_username(
        username: String,
        pool: Pool<Postgres>,
    ) -> Result<Option<Self>, Error> {
        let user = sqlx::query_as::<_, User>(
            "
        SELECT *
        FROM users
        WHERE username = $1;
        ",
        )
        .bind(username)
        .fetch_optional(&pool)
        .await;

        // TODO: Error handling - currently returning sqlx error for simplicity, but we should wrap with our own error type or other error handling
        match user {
            Ok(optional_user) => Ok(optional_user),
            Err(e) => Err(e),
        }
    }

    async fn verify_password(&self, password: &[u8]) -> Result<(), Error> {
        match Argon2::default()
            .verify_password(password, &PasswordHash::new(&self.pass_hash).unwrap())
            .is_ok()
        {
            true => Ok(()),
            false => Err(Error::RowNotFound),
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
async fn main() {
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
        .await
        .unwrap();

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<UserProvidedInfo>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let user = match User::find_by_username(payload.username, pool.clone())
        .await
        .unwrap()
    {
        Some(user) => user,
        None => {
            return Err((
                StatusCode::FORBIDDEN,
                "Wrong username or password".to_string(),
            ));
        }
    };

    user.verify_password(&payload.password.into_bytes())
        .await
        .unwrap();

    Ok(Json(json!(user)))
}

async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<UserProvidedInfo>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let phc = hash_password(&payload.password).unwrap();

    // TODO: Sanitization of payload
    User::create(payload.username, phc, pool)
        .await
        .map_err(internal_error)
        .map(|user| Json(json!(user)))
}

fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // TODO: Tune params
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

// Utility function for mapping any error into a `500 Internal Server Error` response.
// from https://github.com/tokio-rs/axum/blob/main/examples/sqlx-postgres/src/main.rs
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
