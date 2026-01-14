use argon2::Params;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AuthServerState {
    pub pool: PgPool,
    pub argon_params: Params,
    // TODO: add pepper
    // pub pepper: Arc<[u8]>,
}
