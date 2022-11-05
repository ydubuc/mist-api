use axum::{extract::State, http::StatusCode, Json};
use validator::Validate;

use crate::{
    app::models::{api_error::ApiError, json_from_request::JsonFromRequest},
    devices::dtos::{logout_device_dto::LogoutDeviceDto, refresh_device_dto::RefreshDeviceDto},
    AppState,
};

use super::{
    dtos::{login_dto::LoginDto, register_dto::RegisterDto},
    models::access_info::AccessInfo,
    service,
};

pub async fn register(
    State(state): State<AppState>,
    JsonFromRequest(dto): JsonFromRequest<RegisterDto>,
) -> Result<Json<AccessInfo>, ApiError> {
    match dto.validate() {
        Ok(_) => match service::register(&dto, &state.pool).await {
            Ok(user) => Ok(Json(user)),
            Err(e) => Err(e),
        },
        Err(e) => Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        }),
    }
}

pub async fn login(
    State(state): State<AppState>,
    JsonFromRequest(dto): JsonFromRequest<LoginDto>,
) -> Result<Json<AccessInfo>, ApiError> {
    match dto.validate() {
        Ok(_) => match service::login(&dto, &state.pool).await {
            Ok(user) => Ok(Json(user)),
            Err(e) => Err(e),
        },
        Err(e) => Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        }),
    }
}

pub async fn refresh(
    State(state): State<AppState>,
    JsonFromRequest(dto): JsonFromRequest<RefreshDeviceDto>,
) -> Result<Json<AccessInfo>, ApiError> {
    match dto.validate() {
        Ok(_) => match service::refresh(&dto, &state.pool).await {
            Ok(access_info) => Ok(Json(access_info)),
            Err(e) => Err(e),
        },
        Err(e) => Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        }),
    }
}

pub async fn logout(
    State(state): State<AppState>,
    JsonFromRequest(dto): JsonFromRequest<LogoutDeviceDto>,
) -> Result<(), ApiError> {
    match dto.validate() {
        Ok(_) => service::logout(&dto, &state.pool).await,
        Err(e) => Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        }),
    }
}
