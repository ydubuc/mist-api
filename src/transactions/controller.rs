use std::sync::Arc;

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use reqwest::StatusCode;

use crate::{
    app::{models::api_error::ApiError, structs::json_from_request::JsonFromRequest},
    AppState,
};

use super::{service, structs::revenuecat_webbook::RevenueCatWebhook};

pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(webhook): JsonFromRequest<RevenueCatWebhook>,
) -> Result<(), ApiError> {
    if authorization.0.token() != state.envy.revenuecat_webhook_secret {
        return Err(ApiError {
            code: StatusCode::UNAUTHORIZED,
            message: "Invalid authorization".to_string(),
        });
    }

    return service::handle_webhook(webhook, &state).await;
}
