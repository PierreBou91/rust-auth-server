use std::time::Duration;

use axum::{Router, routing::post};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::trace::TraceLayer;

mod error;
mod handlers;
mod session;
mod user;

use crate::{
    error::Result,
    handlers::{login, register},
    user::UserPublic,
};

#[derive(Serialize, Deserialize)]
struct UserProvidedInfo {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // RUST_LOG=debug cargo run
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // TODO: Remove hardcoded DB url
    // TODO: Retry and log if connection fails
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    // TODO: retry and log if connection fails
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

    // TODO: retry and log if listener fails
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    // TODO: retry and log if serve fails
    axum::serve(listener, app).await?;

    Ok(())
}
