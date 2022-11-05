use argon2::{
    password_hash::{self, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use tokio::task;

use crate::app::models::app_error::AppError;

pub async fn hash(password: String) -> Result<String, AppError> {
    let task_result = task::spawn_blocking(move || {
        let salt = SaltString::generate(rand::thread_rng());

        match Argon2::default().hash_password(password.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(_) => Err(AppError {
                message: "Failed to hash password.".to_string(),
            }),
        }
    })
    .await;

    match task_result {
        Ok(result) => result,
        Err(_) => Err(AppError {
            message: "Hash task failed.".to_string(),
        }),
    }
}

pub async fn verify(password: String, hash: String) -> Result<bool, AppError> {
    let task_result = task::spawn_blocking(move || match PasswordHash::new(&hash) {
        Ok(hash) => match Argon2::default().verify_password(password.as_bytes(), &hash) {
            Ok(_) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(_) => Err(AppError {
                message: "Failed to verify password".to_string(),
            }),
        },
        Err(_) => Err(AppError {
            message: "Invalid hash.".to_string(),
        }),
    })
    .await;

    match task_result {
        Ok(result) => result,
        Err(_) => Err(AppError {
            message: "Verify task failed.".to_string(),
        }),
    }
}
