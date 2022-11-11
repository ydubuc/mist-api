use crate::app::models::api_error::ApiError;
use axum::Json;
use axum_macros::FromRequest;

#[derive(FromRequest)]
#[from_request(via(Json), rejection(ApiError))]
pub struct JsonFromRequest<T>(pub T);
