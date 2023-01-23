use std::sync::Arc;

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    Json, TypedHeader,
};
use axum_macros::debug_handler;

use crate::{
    app::{models::api_error::ApiError, structs::json_from_request::JsonFromRequest},
    media::modal::dtos::receive_webhook_dto::ReceiveWebhookDto,
    AppState,
};

#[debug_handler]
pub async fn receive_webhook(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<ReceiveWebhookDto>,
) -> Result<Json<String>, ApiError> {
    tracing::debug!("{:?}", authorization);
    tracing::debug!("{:?}", dto.request_id);
    tracing::debug!("{:?}", dto.images);

    return Ok(Json("received".to_string()));
}
