// Error handling
// from Jeremy Chone excellent video on error handling
// https://www.youtube.com/watch?v=j-VQCYP6wyw

use axum::{Json, http::StatusCode, response::IntoResponse};
use derive_more::From;
use serde_json::json;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    UserAlreadyExists,
    WrongCredentials,

    #[from]
    Sqlx(sqlx::Error),

    #[from]
    Argon2(argon2::password_hash::Error),

    #[from]
    Io(std::io::Error),

    #[from]
    TokioJoin(tokio::task::JoinError),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // You can improve this later (thiserror), but this is fine for now.
        write!(f, "{self:?}")
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        // Log internal details (server-side)
        match &self {
            Error::Sqlx(e) => tracing::error!(error = ?e, "db error"),
            Error::Argon2(e) => tracing::error!(error = ?e, "argon2 error"),
            Error::Io(e) => tracing::error!(error = ?e, "io error"),
            Error::TokioJoin(e) => tracing::error!(error = ?e, "tokio join error"),
            _ => tracing::warn!(error = ?self, "request failed"),
        }

        // Return safe messages to clients
        match self {
            Error::UserAlreadyExists => (
                StatusCode::CONFLICT,
                Json(json!({ "error": "user already exists" })),
            )
                .into_response(),

            Error::WrongCredentials => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "wrong credentials" })),
            )
                .into_response(),

            Error::Sqlx(_) | Error::Argon2(_) | Error::Io(_) | Error::TokioJoin(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal server error" })),
            )
                .into_response(),
        }
    }
}
