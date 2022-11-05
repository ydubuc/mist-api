use axum::http::StatusCode;

use crate::app::models::api_error::ApiError;

#[derive(Debug)]
pub enum UsersApiError {
    UserNotFound,
    PermissionDenied,
}

impl UsersApiError {
    pub fn value(&self) -> ApiError {
        match *self {
            Self::UserNotFound => ApiError {
                code: StatusCode::NOT_FOUND,
                message: "User not found.".to_string(),
            },
            Self::PermissionDenied => ApiError {
                code: StatusCode::UNAUTHORIZED,
                message: "Permission denied.".to_string(),
            },
        }
    }
}
