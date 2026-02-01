use std::sync::Arc;

use crate::service::user::UserService;

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub env: Env,
}

#[derive(Clone)]
pub enum Env {
    Prod,
    Dev,
}
