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

use super::{service, structs::revenue_cat_webbook::RevenueCatWebhook};

pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(webhook): JsonFromRequest<RevenueCatWebhook>,
) -> Result<(), ApiError> {
    println!("received webhook header {:?}", authorization);
    println!("received webhook {:?}", webhook);

    if authorization.0.token() != state.envy.revenuecat_webhook_secret {
        println!("not valid token");
        return Err(ApiError {
            code: StatusCode::UNAUTHORIZED,
            message: "Invalid authorization".to_string(),
        });
    }

    return service::handle_webhook(webhook, &state).await;
}
