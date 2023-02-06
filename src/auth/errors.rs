use axum::http::StatusCode;

use crate::app::models::api_error::ApiError;

#[derive(Debug)]
pub enum AuthApiError {
    BadLogin,
}

impl AuthApiError {
    pub fn value(&self) -> ApiError {
        match *self {
            Self::BadLogin => ApiError {
                code: StatusCode::UNAUTHORIZED,
                message: "Invalid username or password.".to_string(),
            },
        }
    }
}
