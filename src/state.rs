use sqlx::PgPool;

#[derive(Clone)]
pub struct AuthServerState {
    pub pool: PgPool,
}
