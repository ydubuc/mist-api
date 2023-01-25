use std::sync::Arc;

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use axum_macros::debug_handler;
use reqwest::StatusCode;

use crate::{
    app::{
        errors::DefaultApiError, models::api_error::ApiError,
        structs::json_from_request::JsonFromRequest,
    },
    generate_media_requests, media,
    webhooks::modal::dtos::receive_webhook_dto::ReceiveWebhookDto,
    AppState,
};

#[debug_handler]
pub async fn receive_webhook(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<ReceiveWebhookDto>,
) -> Result<(), ApiError> {
    if authorization.0.token() != state.envy.modal_webhook_secret {
        return Err(DefaultApiError::PermissionDenied.value());
    }

    let id = &dto.request_id;

    match generate_media_requests::service::get_generate_media_request_by_id_as_admin(
        id,
        &state.pool,
    )
    .await
    {
        Ok(request) => media::apis::modal::service::on_receive_webhook(request, dto, state),
        Err(e) => {
            if e.code == StatusCode::NOT_FOUND {
                tracing::error!("receive_webhook failed: request not found");
                return Ok(());
            } else {
                return Err(DefaultApiError::InternalServerError.value());
            }
        }
    }

    return Ok(());
}
