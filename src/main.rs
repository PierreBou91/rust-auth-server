use std::{sync::Arc, time::Duration};

use argon2::ParamsBuilder;
use axum::{
    Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;

mod config;
mod error;
mod handlers;
mod session;
mod state;
mod user;

use crate::{
    config::Config,
    error::Result,
    handlers::{health_check, login, register},
    state::AuthServerState,
    user::UserPublic,
};

#[derive(Serialize, Deserialize)]
struct UserProvidedInfo {
    username: String,
    password: String,
}

type AppState = Arc<AuthServerState>;

#[tokio::main]
async fn main() -> Result<()> {
    // RUST_LOG=debug cargo run
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();

    let mut builder = ParamsBuilder::new();
    let argon_params = builder
        .m_cost(config.argon2_memory)
        .t_cost(config.argon2_iteration)
        .p_cost(config.argon2_parallelism)
        .output_len(32)
        .build()
        .unwrap();

    // let argon = Argon2::from(argon_param);

    // TODO: retry and log if connection fails
    let pool = PgPoolOptions::new()
        .max_connections(config.max_db_connections)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await?;

    let state: AppState = Arc::new(AuthServerState { pool, argon_params });

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // TODO: retry and log if listener fails
    let listener =
        tokio::net::TcpListener::bind(format!("{:}:{:}", config.server_host, config.server_port))
            .await?;
    // TODO: retry and log if serve fails
    axum::serve(listener, app).await?;

    Ok(())
}
