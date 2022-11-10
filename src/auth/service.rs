use axum::http::StatusCode;
use jsonwebtoken::errors::ErrorKind;
use sqlx::PgPool;

use crate::{
    app::{errors::DefaultApiError, models::api_error::ApiError, util::hasher},
    devices::{
        self,
        dtos::{
            get_devices_filter_dto::GetDevicesFilterDto, logout_device_dto::LogoutDeviceDto,
            refresh_device_dto::RefreshDeviceDto,
        },
        models::device::Device,
    },
    mail::{self, templates::request_password_update_template::request_password_update_template},
    users,
};

use super::{
    dtos::{
        edit_password_dto::EditPasswordDto, login_dto::LoginDto, register_dto::RegisterDto,
        request_password_update_dto::RequestPasswordUpdateDto,
    },
    jwt::{
        enums::pepper_type::PepperType,
        models::claims::Claims,
        util::{decode_jwt, sign_jwt},
    },
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
                    access_token: sign_jwt(&user.id, None),
                    refresh_token: Some(device.refresh_token),
                    device_id: Some(device.id),
                }),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn request_password_update_mail(
    dto: &RequestPasswordUpdateDto,
    pool: &PgPool,
) -> Result<(), ApiError> {
    match users::service::get_user_by_email_as_admin(&dto.email, pool).await {
        Ok(user) => {
            tokio::spawn(async move {
                let access_token = sign_jwt(&user.id, Some(PepperType::EDIT_PASSWORD));
                let template = request_password_update_template(&user, &access_token);
                mail::service::send_mail(&user.email, &template.0, &template.1).await
            });

            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub async fn process_password_edit(
    access_token: &str,
    dto: &EditPasswordDto,
    pool: &PgPool,
) -> Result<(), ApiError> {
    match decode_jwt(access_token.to_string(), Some(PepperType::EDIT_PASSWORD)) {
        Ok(claims) => {
            users::service::edit_user_password_by_id_as_admin(&claims.id, dto, pool).await
        }
        Err(e) => match e {
            ErrorKind::ExpiredSignature => Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: "Token has expired.".to_string(),
            }),
            _ => Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: "Invalid token.".to_string(),
            }),
        },
    }
}

pub async fn get_devices(
    dto: &GetDevicesFilterDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<Vec<Device>, ApiError> {
    return devices::service::get_devices(dto, claims, pool).await;
}

pub async fn refresh(dto: &RefreshDeviceDto, pool: &PgPool) -> Result<AccessInfo, ApiError> {
    match devices::service::refresh_device_as_admin(dto, pool).await {
        Ok(_) => Ok(AccessInfo {
            access_token: sign_jwt(&dto.user_id, None),
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
