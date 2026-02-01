use std::sync::Arc;

// use crate::crypto::password::{hash_password_async, verify_password_async};
use crate::crypto::password::hash_password_async;
use crate::domain::{self, User};
use crate::error::Result;
// use crate::error::{Error, Result};
use crate::repo::{UserRecord, UserRepo};
use argon2::Params;
// use sqlx::PgPool;
// use tracing::instrument;

#[derive(Clone)]
pub struct UserService {
    repo: Arc<dyn UserRepo>,
    pub argon_params: Params,
    // TODO: add pepper
    // pub pepper: Arc<[u8]>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepo>, argon_params: Params) -> Self {
        Self { repo, argon_params }
    }
    pub async fn register(&self, username: &str, password: &str) -> Result<domain::User> {
        let hashed_password =
            hash_password_async(password.to_string(), self.argon_params.clone()).await?;
        let user = self.repo.insert_user(username, &hashed_password).await?;
        Ok(User::from(user))
    }
}

impl From<UserRecord> for User {
    fn from(value: UserRecord) -> Self {
        Self {
            id: value.id,
            username: value.username,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

// const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$pmcRQCyDK/zHE03SpG7d2A$/204Eb0JCZ72yrZ+CUFRCv0X91nkLyNix8lQxRFyD5g";

// #[instrument(skip(password, pool))]
// pub async fn authenticate_user(
//     username: &str,
//     password: &str,
//     pool: &PgPool,
// ) -> Result<domain::User> {
//     let user_opt = User::find_by_username(username, pool).await?;

//     // Always run verification to equalize timing
//     let phc: String = match &user_opt {
//         Some(user) => user.pass_hash.clone(),
//         None => DUMMY_HASH.to_string(),
//     };

//     let verify_result = verify_password_async(phc, password.as_bytes().to_vec()).await;

//     let Some(user) = user_opt else {
//         return Err(Error::WrongCredentials);
//     };

//     verify_result.map_err(|_| Error::WrongCredentials)?;

//     Ok(UserPublic::from(user))
// }

// pub fn create_user() -> Result<domain::User> {
//     // Hash password
//     // Add user to DB

//     Ok(domain::User {
//         id: todo!(),
//         username: todo!(),
//         created_at: todo!(),
//         updated_at: todo!(),
//     })
// }
