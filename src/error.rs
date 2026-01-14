// Error handling
// from Jeremy Chone excellent video on error handling
// https://www.youtube.com/watch?v=j-VQCYP6wyw

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;
use tracing::{error, warn};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Wrong credentials")]
    WrongCredentials,

    #[error("Error when accessing the database")]
    Sqlx(#[from] sqlx::Error),

    #[error("Error hashing password")]
    Argon2(#[from] argon2::password_hash::Error),

    #[error("Error from rand")]
    OsRng(#[from] rand::rand_core::OsError),

    #[error("Io error")]
    Io(#[from] std::io::Error),

    #[error("Error joining Tokio tasks")]
    TokioJoin(#[from] tokio::task::JoinError),
}

impl Error {
    fn log(&self) {
        match self {
            Error::UserAlreadyExists => warn!(error = %self, "client error"),
            Error::WrongCredentials => warn!(error = %self, "client error"),
            _ => error!(error = %self, details = ?self, "server error"),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        self.log();
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

            Error::OsRng(_)
            | Error::Sqlx(_)
            | Error::Argon2(_)
            | Error::Io(_)
            | Error::TokioJoin(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal server error" })),
            )
                .into_response(),
        }
    }
}
