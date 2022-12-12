use axum::http::StatusCode;

use super::models::api_error::ApiError;

#[derive(Debug)]
pub enum DefaultApiError {
    InternalServerError,
    PermissionDenied,
}

impl DefaultApiError {
    pub fn value(&self) -> ApiError {
        match *self {
            Self::InternalServerError => ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "An internal server error occurred.".to_string(),
            },
            Self::PermissionDenied => ApiError {
                code: StatusCode::UNAUTHORIZED,
                message: "Permission denied.".to_string(),
            },
        }
    }
}
