use super::*;
use axum::{Json, extract::State};

pub async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<UserProvidedInfo>,
) -> axum::response::Result<Json<UserPublic>> {
    // TODO: Sanitization of payload
    let user = user::create_user(&payload.username, &payload.password, &pool).await?;
    Ok(Json(user))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<UserProvidedInfo>,
) -> axum::response::Result<Json<UserPublic>> {
    let user = user::authenticate_user(&payload.username, &payload.password, &pool).await?;
    Ok(Json(user))
}
