use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    app::{errors::DefaultApiError, models::api_error::ApiError, util::hasher},
    devices::{
        self,
        dtos::{logout_device_dto::LogoutDeviceDto, refresh_device_dto::RefreshDeviceDto},
    },
    users,
};

use super::{
    dtos::{login_dto::LoginDto, register_dto::RegisterDto},
    jwt::util::sign_jwt,
    models::access_info::AccessInfo,
};

pub async fn register(dto: &RegisterDto, pool: &PgPool) -> Result<AccessInfo, ApiError> {
    match users::service::create_user_as_admin(dto, pool).await {
        Ok(_) => {
            let login_dto = LoginDto {
                username: None,
                email: Some(dto.email.to_string()),
                password: dto.password.to_string(),
            };

            return login(&login_dto, &pool).await;
        }
        Err(e) => Err(e),
    }
}

pub async fn login(dto: &LoginDto, pool: &PgPool) -> Result<AccessInfo, ApiError> {
    match users::service::get_user_by_login_dto_as_admin(dto, pool).await {
        Ok(user) => {
            let Ok(matches) = hasher::verify(dto.password.to_string(), user.password_hash.to_string()).await
            else {
                return Err(DefaultApiError::InternalServerError.value());
            };

            if !matches {
                return Err(ApiError {
                    code: StatusCode::UNAUTHORIZED,
                    message: "Invalid password.".to_string(),
                });
            }

            match devices::service::create_device_as_admin(&user, pool).await {
                Ok(device) => Ok(AccessInfo {
                    access_token: sign_jwt(&user.id),
                    refresh_token: Some(device.refresh_token),
                    device_id: Some(device.id),
                }),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn refresh(dto: &RefreshDeviceDto, pool: &PgPool) -> Result<AccessInfo, ApiError> {
    match devices::service::refresh_device_as_admin(dto, pool).await {
        Ok(_) => Ok(AccessInfo {
            access_token: sign_jwt(&dto.user_id),
            refresh_token: None,
            device_id: None,
        }),
        Err(e) => Err(e),
    }
}

pub async fn logout(dto: &LogoutDeviceDto, pool: &PgPool) -> Result<(), ApiError> {
    match devices::service::logout_device_as_admin(dto, pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
