use axum::http::StatusCode;

use crate::app::models::api_error::ApiError;

#[derive(Debug)]
pub enum MediaApiError {
    MediaNotFound,
}

impl MediaApiError {
    pub fn value(&self) -> ApiError {
        match *self {
            Self::MediaNotFound => ApiError {
                code: StatusCode::NOT_FOUND,
                message: "Media not found.".to_string(),
            },
        }
    }
}
