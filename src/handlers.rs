use super::*;

use axum::{Json, extract::State, http::StatusCode};
use tracing::{debug, instrument};

#[instrument(skip(state, payload), fields(username = %payload.username))]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<UserProvidedInfo>,
) -> axum::response::Result<Json<UserPublic>> {
    debug!("register attempt");
    // TODO: Sanitization of payload
    let user = user::create_user(
        &payload.username,
        &payload.password,
        &state.pool,
        state.argon_params.clone(),
    )
    .await?;

    debug!("register success");
    Ok(Json(user))
}

#[instrument(skip(state, payload), fields(username = %payload.username))]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<UserProvidedInfo>,
) -> axum::response::Result<Json<UserPublic>> {
    tracing::debug!("login attempt");
    let user = user::authenticate_user(&payload.username, &payload.password, &state.pool).await?;

    tracing::debug!("login success");
    Ok(Json(user))
}

#[instrument(skip(state))]
pub async fn health_check(State(state): State<AppState>) -> StatusCode {
    match user::health_check(&state.pool).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
