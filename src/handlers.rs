use super::*;

use axum::{Json, extract::State, http::StatusCode};
use tracing::instrument;

#[instrument(skip(state, payload), fields(username = %payload.username))]
pub async fn register(
    State(state): State<AuthServerState>,
    Json(payload): Json<UserProvidedInfo>,
) -> axum::response::Result<Json<UserPublic>> {
    // TODO: Sanitization of payload
    let user = user::create_user(&payload.username, &payload.password, &state.pool).await?;
    Ok(Json(user))
}

#[instrument(skip(state, payload), fields(username = %payload.username))]
pub async fn login(
    State(state): State<AuthServerState>,
    Json(payload): Json<UserProvidedInfo>,
) -> axum::response::Result<Json<UserPublic>> {
    tracing::info!("Logging in user: {}", payload.username);
    let user = user::authenticate_user(&payload.username, &payload.password, &state.pool).await?;
    Ok(Json(user))
}

#[instrument(skip(state))]
pub async fn health_check(State(state): State<AuthServerState>) -> StatusCode {
    match user::health_check(&state.pool).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
