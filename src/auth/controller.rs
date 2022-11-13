use axum::{
    extract::{Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use validator::Validate;

use crate::{
    app::{models::api_error::ApiError, structs::json_from_request::JsonFromRequest},
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
        request_email_update_dto::RequestEmailUpdateDto,
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
    if let Err(e) = dto.validate() {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        });
    }

    match service::register(&dto, &state).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e),
    }
}

pub async fn login(
    State(state): State<AppState>,
    JsonFromRequest(dto): JsonFromRequest<LoginDto>,
) -> Result<Json<AccessInfo>, ApiError> {
    if let Err(e) = dto.validate() {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        });
    }

    match service::login(&dto, &state).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e),
    }
}

pub async fn request_email_update_mail(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<RequestEmailUpdateDto>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => service::request_email_update_mail(&dto, &claims, &state).await,
        Err(e) => Err(e),
    }
}

pub async fn process_email_edit(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    let access_token = authorization.0.token();
    service::process_email_edit(&access_token, &state).await
}

pub async fn request_password_update_mail(
    State(state): State<AppState>,
    JsonFromRequest(dto): JsonFromRequest<RequestPasswordUpdateDto>,
) -> Result<(), ApiError> {
    if let Err(e) = dto.validate() {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        });
    }

    service::request_password_update_mail(&dto, &state).await
}

pub async fn process_password_edit(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<EditPasswordDto>,
) -> Result<(), ApiError> {
    if let Err(e) = dto.validate() {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        });
    }

    let access_token = authorization.0.token();
    service::process_password_edit(&access_token, &dto, &state).await
}

pub async fn get_devices(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetDevicesFilterDto>,
) -> Result<Json<Vec<Device>>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::get_devices(&dto, &claims, &state.pool).await {
                Ok(posts) => Ok(Json(posts)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn refresh(
    State(state): State<AppState>,
    JsonFromRequest(dto): JsonFromRequest<RefreshDeviceDto>,
) -> Result<Json<AccessInfo>, ApiError> {
    if let Err(e) = dto.validate() {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        });
    }

    match service::refresh(&dto, &state).await {
        Ok(access_info) => Ok(Json(access_info)),
        Err(e) => Err(e),
    }
}

pub async fn logout(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<LogoutDeviceDto>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            service::logout(&dto, &claims, &state.pool).await
        }
        Err(e) => Err(e),
    }
}
