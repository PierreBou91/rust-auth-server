use crate::app::AppState;
use crate::error::Result;

use axum::{
    Json,
    extract::State,
    // http::{HeaderMap, StatusCode, header::SET_COOKIE},
    // response::{AppendHeaders, IntoResponse},
};
// use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
// use chrono::Utc;
// use serde_json::json;
// use tracing::instrument;

use super::dto::{PublicUser, RegisterUser};

// #[instrument(skip(state, payload), fields(username = %payload.username))]
// pub async fn register(
//     State(state): State<AppState>,
//     Json(payload): Json<UserProvidedInfo>,
// ) -> axum::response::Result<Json<UserPublic>> {
//     debug!("register attempt");
//     // TODO: Sanitization of payload
//     let user = user::create_user(
//         &payload.username,
//         &payload.password,
//         &state.pool,
//         state.argon_params.clone(),
//     )
//     .await?;

//     debug!("register success");
//     Ok(Json(user))
// }

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<PublicUser>> {
    let user = state
        .user_service
        .register(&payload.username, &payload.password)
        .await?;

    Ok(Json(PublicUser::from(user)))
}

// #[instrument(skip(state, payload), fields(username = %payload.username))]
// pub async fn login(
//     State(state): State<AppState>,
//     Json(payload): Json<UserProvidedInfo>,
// ) -> Result<impl IntoResponse> {
//     tracing::debug!("login attempt");
//     let user = user::authenticate_user(&payload.username, &payload.password, &state.pool).await?;
//     tracing::debug!("login success");
//     tracing::debug!("create session attemp");
//     let sess = session::create_session(&user.id, &state.pool).await?;
//     let b64sess = BASE64_URL_SAFE_NO_PAD.encode(&sess.0);
//     let now = Utc::now();
//     let max_age = (sess.1 - now).num_seconds();
//     assert!(max_age > 0);
//     let secure = match state.env {
//         state::Env::Prod => "Secure; ",
//         state::Env::Dev => "",
//     };
//     let header = AppendHeaders([(
//         SET_COOKIE,
//         format!(
//             "session={:}; Max-Age={:}; HttpOnly; {:}Path=/; SameSite=Lax",
//             b64sess, max_age, secure
//         ),
//     )]);

//     Ok((header, Json(user)))
// }

// // #[derive(Deserialize)]
// // struct AuthenticatedUser {}

// #[instrument(skip(state))]
// pub async fn me(headers: HeaderMap, State(state): State<AppState>) -> Result<impl IntoResponse> {
//     let session_id = headers.get("session");
//     if let Some(sess_id) = session_id {
//         let session_id = BASE64_URL_SAFE_NO_PAD.decode(sess_id)?;
//         let session = find_session_by_token(&session_id, &state.pool).await?;
//         if let Some(session_from_db) = session {
//             Ok(Json(json!(
//                 find_user_by_id(&session_from_db.user_id, &state.pool).await?
//             )))
//         } else {
//             Err(error::Error::Unauthenticated)
//         }
//     } else {
//         Err(error::Error::Unauthenticated)
//     }
// }

// #[instrument(skip(state))]
// pub async fn health_check(State(state): State<AppState>) -> StatusCode {
//     match user::health_check(&state.pool).await {
//         Ok(_) => StatusCode::OK,
//         Err(_) => StatusCode::SERVICE_UNAVAILABLE,
//     }
// }
