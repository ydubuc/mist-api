use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use validator::Validate;

use crate::{
    app::models::{api_error::ApiError, json_from_request::JsonFromRequest},
    devices::{
        dtos::{
            get_devices_filter_dto::GetDevicesFilterDto, logout_device_dto::LogoutDeviceDto,
            refresh_device_dto::RefreshDeviceDto,
        },
        models::device::Device,
    },
    AppState,
};

use super::{
    dtos::{
        edit_password_dto::EditPasswordDto, login_dto::LoginDto, register_dto::RegisterDto,
        request_password_update_dto::RequestPasswordUpdateDto,
    },
    jwt::models::claims::Claims,
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

pub async fn request_password_update_mail(
    State(state): State<AppState>,
    JsonFromRequest(dto): JsonFromRequest<RequestPasswordUpdateDto>,
) -> Result<(), ApiError> {
    match dto.validate() {
        Ok(_) => service::request_password_update_mail(&dto, &state.pool).await,
        Err(e) => Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        }),
    }
}

pub async fn process_password_edit(
    State(state): State<AppState>,
    Path(access_token): Path<String>,
    JsonFromRequest(dto): JsonFromRequest<EditPasswordDto>,
) -> Result<(), ApiError> {
    match dto.validate() {
        Ok(_) => service::process_password_edit(&access_token, &dto, &state.pool).await,
        Err(e) => Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        }),
    }
}

pub async fn get_devices(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetDevicesFilterDto>,
) -> Result<Json<Vec<Device>>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match dto.validate() {
            Ok(_) => match service::get_devices(&dto, &claims, &state.pool).await {
                Ok(posts) => Ok(Json(posts)),
                Err(e) => Err(e),
            },
            Err(e) => Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: e.to_string(),
            }),
        },
        Err(e) => Err(e),
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
