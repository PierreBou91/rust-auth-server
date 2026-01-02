use std::time::Duration;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct CreateUser {
    username: String,
    pass_hash: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct UserRecord {
    id: Uuid,
    username: String,
    pass_hash: String,
    created_at: DateTime<Local>,
    updated_at: DateTime<Local>,
}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .unwrap();

    // build our application with a single route
    let app = Router::new()
        .route("/create_user", post(create_user))
        .with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Value>, (StatusCode, String)> {
    sqlx::query_as::<_, UserRecord>(
        "
        INSERT INTO
        users (username,pass_hash)
        VALUES ($1,$2)
        RETURNING *;
        ",
    )
    .bind(payload.username)
    .bind(payload.pass_hash)
    .fetch_one(&pool)
    .await
    .map_err(internal_error)
    .map(|user| Json(json!(user)))
}

// Utility function for mapping any error into a `500 Internal Server Error`
// response.
// from https://github.com/tokio-rs/axum/blob/main/examples/sqlx-postgres/src/main.rs
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
