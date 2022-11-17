use reqwest::StatusCode;

use crate::app::models::api_error::ApiError;

#[derive(Debug)]
pub enum HandlersApiError {
    TransactionError,
    TransactionFailure,
    ProductNotImplemented,
}

impl HandlersApiError {
    pub fn value(&self) -> ApiError {
        match *self {
            Self::TransactionError => ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Database transaction failed.".to_string(),
            },
            Self::TransactionFailure => ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Database transaction did not go through.".to_string(),
            },
            Self::ProductNotImplemented => ApiError {
                code: StatusCode::NOT_IMPLEMENTED,
                message: "Product Id not implemented.".to_string(),
            },
        }
    }
}
