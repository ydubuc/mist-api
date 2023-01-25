use axum::http::StatusCode;

use crate::app::models::api_error::ApiError;

#[derive(Debug)]
pub enum GenerateMediaRequestsApiError {
    RequestNotFound,
}

impl GenerateMediaRequestsApiError {
    pub fn value(&self) -> ApiError {
        match *self {
            Self::RequestNotFound => ApiError {
                code: StatusCode::NOT_FOUND,
                message: "Request not found.".to_string(),
            },
        }
    }
}
