use axum::http::StatusCode;

use crate::app::models::api_error::ApiError;

#[derive(Debug)]
pub enum PostsApiError {
    PostNotFound,
}

impl PostsApiError {
    pub fn value(&self) -> ApiError {
        match *self {
            Self::PostNotFound => ApiError {
                code: StatusCode::NOT_FOUND,
                message: "Post not found.".to_string(),
            },
        }
    }
}
