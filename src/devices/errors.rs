use axum::http::StatusCode;

use crate::app::models::api_error::ApiError;

#[derive(Debug)]
pub enum DevicesApiError {
    DeviceNotFound,
}

impl DevicesApiError {
    pub fn value(&self) -> ApiError {
        match *self {
            Self::DeviceNotFound => ApiError {
                code: StatusCode::NOT_FOUND,
                message: "Device not found.".to_string(),
            },
        }
    }
}
