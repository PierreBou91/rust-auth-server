use crate::error::Result;
use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

fn hash_password(password: &str, argon_params: Params) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon_params,
    );
    let hashed_password = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hashed_password.to_string())
}

pub async fn hash_password_async(password: String, argon_params: Params) -> Result<String> {
    let phc = tokio::task::spawn_blocking(move || hash_password(&password, argon_params)).await??;
    Ok(phc)
}

fn verify_password(phc: &str, password: &[u8]) -> Result<()> {
    let parsed = PasswordHash::new(phc)?;
    // Params are not required for the verification because they are extracted from the phc
    Argon2::default().verify_password(password, &parsed)?;
    Ok(())
}

pub async fn verify_password_async(phc: String, password: Vec<u8>) -> Result<()> {
    tokio::task::spawn_blocking(move || verify_password(&phc, &password)).await??;
    Ok(())
}
