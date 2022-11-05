use axum::Json;
use axum_macros::FromRequest;

use super::api_error::ApiError;

#[derive(FromRequest)]
#[from_request(via(Json), rejection(ApiError))]
pub struct JsonFromRequest<T>(pub T);
