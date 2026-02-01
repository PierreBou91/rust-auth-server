use sqlx::PgPool;

use super::*;

pub struct PgSessionRepo {
    pub pool: PgPool,
}

#[async_trait]
impl SessionRepo for PgSessionRepo {
    async fn select_by_token(&self, _token: &str) -> Result<SessionRecord> {
        todo!()
    }

    async fn insert_session(&self, _session: SessionRecord) -> Result<SessionRecord> {
        todo!()
    }
}
