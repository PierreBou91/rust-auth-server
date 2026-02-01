use axum::{
    Router,
    // routing::{get, post},
    routing::post,
};
use tower_http::trace::TraceLayer;

use crate::{
    // http::handlers::{health_check, login, me, register},
    http::handlers::register,
};

pub mod state;
pub use state::AppState;

pub fn init_router(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        // .route("/login", post(login))
        // .route("/health", get(health_check))
        // .route("/me", get(me))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
