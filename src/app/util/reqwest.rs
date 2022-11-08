use bytes::Bytes;
use reqwest::StatusCode;

use crate::app::models::api_error::ApiError;

pub async fn get_bytes(url: &str) -> Result<Bytes, ApiError> {
    match reqwest::get(url).await {
        Ok(res) => match res.bytes().await {
            Ok(bytes) => Ok(bytes),
            Err(e) => {
                tracing::error!(%e);
                Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "Failed to get bytes from response.".to_string(),
                })
            }
        },
        Err(e) => {
            tracing::error!(%e);
            Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to get url response.".to_string(),
            })
        }
    }
}
