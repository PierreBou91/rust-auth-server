use sqlx::PgPool;

use crate::error::Error;

use super::*;

pub struct PgUserRepo {
    pub pool: PgPool,
}

#[async_trait]
impl UserRepo for PgUserRepo {
    async fn insert_user(&self, username: &str, hashed_password: &str) -> Result<UserRecord> {
        let user: UserRecord = sqlx::query_as::<_, UserRecord>(
            r#"
            INSERT INTO users (username, pass_hash)
            VALUES ($1, $2)
            RETURNING id, username, pass_hash, created_at, updated_at
            "#,
        )
        .bind(username)
        .bind(hashed_password)
        .fetch_one(&self.pool)
        .await
        .map_err(map_unique_violation)?;

        Ok(user)
    }

    async fn select_by_username(&self, _username: &str) -> Result<Option<UserRecord>> {
        todo!()
    }
}

fn map_unique_violation(e: sqlx::Error) -> Error {
    if let sqlx::Error::Database(db_err) = &e {
        // Postgres unique_violation is SQLSTATE 23505
        if db_err.code().as_deref() == Some("23505") {
            return Error::UserAlreadyExists;
        }
    }
    Error::Sqlx(e)
}
