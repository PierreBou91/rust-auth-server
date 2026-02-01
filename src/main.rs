use std::{sync::Arc, time::Duration};

use argon2::ParamsBuilder;

use server::{
    app::{self, AppState, state},
    config::Config,
    error::Result,
    repo::pg_user::PgUserRepo,
    service::user::UserService,
};
use sqlx::postgres::PgPoolOptions;
use tracing::debug;

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

    // TODO: retry and log if connection fails
    let pool = PgPoolOptions::new()
        .max_connections(config.max_db_connections)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await?;

    let env = match config.env.to_uppercase().as_str() {
        "DEV" => state::Env::Dev,
        _ => state::Env::Prod,
    };

    let repo = Arc::new(PgUserRepo { pool });

    let user_service = Arc::new(UserService::new(repo, argon_params));

    let state = AppState { env, user_service };

    let app = app::init_router(state);

    // TODO: retry and log if listener fails
    let listener =
        tokio::net::TcpListener::bind(format!("{:}:{:}", config.server_host, config.server_port))
            .await?;

    debug!("Listening on localhost:{:}", config.server_port);

    // TODO: retry and log if serve fails
    axum::serve(listener, app).await?;

    Ok(())
}
