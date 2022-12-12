use std::sync::Arc;

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    Json, TypedHeader,
};
use axum_macros::debug_handler;
use reqwest::StatusCode;
use serde_json::Value;
use validator::Validate;

use crate::{auth::jwt::models::claims::Claims, AppState};

use super::{
    dtos::edit_api_status_dto::EditApiStatusDto, models::api_error::ApiError, service,
    structs::json_from_request::JsonFromRequest,
};

pub async fn get_root(State(_state): State<Arc<AppState>>) -> Result<(), ApiError> {
    Ok(())
}

#[debug_handler]
pub async fn get_api_state(State(state): State<Arc<AppState>>) -> Json<Value> {
    return Json(service::get_api_state(&state).await);
}

pub async fn edit_api_state(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<EditApiStatusDto>,
) -> Result<Json<Value>, ApiError> {
    let authorization_copy = authorization.clone();

    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::edit_api_state(&dto, &claims, &authorization_copy, &state).await {
                Ok(value) => Ok(Json(value)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}
