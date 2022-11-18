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
    mail::{
        self,
        templates::{
            request_email_update_template::request_email_update_template,
            request_password_update_template::request_password_update_template,
        },
    },
    users, AppState,
};

use super::{
    dtos::{
        delete_account_dto::DeleteAccountDto, edit_password_dto::EditPasswordDto,
        login_dto::LoginDto, register_dto::RegisterDto,
        request_email_update_dto::RequestEmailUpdateDto,
        request_password_update_dto::RequestPasswordUpdateDto,
    },
    jwt::{
        enums::pepper_type::PepperType,
        models::claims::Claims,
        util::{decode_jwt, sign_jwt},
    },
    models::access_info::AccessInfo,
};

pub async fn register(dto: &RegisterDto, state: &AppState) -> Result<AccessInfo, ApiError> {
    match users::service::create_user_as_admin(dto, &state.pool).await {
        Ok(_) => {
            let login_dto = LoginDto {
                username: None,
                email: Some(dto.email.to_string()),
                password: dto.password.to_string(),
            };

            return login(&login_dto, state).await;
        }
        Err(e) => Err(e),
    }
}

pub async fn login(dto: &LoginDto, state: &AppState) -> Result<AccessInfo, ApiError> {
    match users::service::get_user_by_login_dto_as_admin(dto, &state.pool).await {
        Ok(user) => {
            if user.delete_pending {
                return Err(ApiError {
                    code: StatusCode::UNAUTHORIZED,
                    message: "This user is being deleted.".to_string(),
                });
            }

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

            match devices::service::create_device_as_admin(&user, &state.pool).await {
                Ok(device) => Ok(AccessInfo {
                    access_token: sign_jwt(&user.id, &state.envy.jwt_secret, None),
                    refresh_token: Some(device.refresh_token),
                    device_id: Some(device.id),
                }),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn request_email_update_mail(
    dto: &RequestEmailUpdateDto,
    claims: &Claims,
    state: &AppState,
) -> Result<(), ApiError> {
    let user_result = users::service::get_user_by_id(&claims.id, claims, &state.pool).await;
    let Ok(user) = user_result
    else {
        return Err(user_result.unwrap_err());
    };

    if user.delete_pending {
        return Err(ApiError {
            code: StatusCode::UNAUTHORIZED,
            message: "This user is being deleted.".to_string(),
        });
    }

    if let Ok(_) = users::service::get_user_by_email_as_admin(&dto.email, &state.pool).await {
        return Err(ApiError {
            code: StatusCode::CONFLICT,
            message: "Email already exists.".to_string(),
        });
    }

    match users::service::edit_user_email_pending_by_id_as_admin(
        &claims.id,
        &dto.email,
        &state.pool,
    )
    .await
    {
        Ok(_) => {
            let envy = state.envy.clone();
            let email = dto.email.clone();
            let id = claims.id.clone();

            tokio::spawn(async move {
                let access_token = sign_jwt(&id, &envy.jwt_secret, Some(PepperType::EDIT_EMAIL));
                let template =
                    request_email_update_template(&user, &access_token, &envy.frontend_url);
                mail::service::send_mail(&email, &template.0, &template.1, &envy).await
            });

            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub async fn process_email_edit(access_token: &str, state: &AppState) -> Result<(), ApiError> {
    match decode_jwt(
        access_token.to_string(),
        &state.envy.jwt_secret,
        Some(PepperType::EDIT_EMAIL),
    ) {
        Ok(claims) => {
            users::service::approve_user_email_pending_by_id_as_admin(&claims.id, &state.pool).await
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

pub async fn request_password_update_mail(
    dto: &RequestPasswordUpdateDto,
    state: &AppState,
) -> Result<(), ApiError> {
    match users::service::get_user_by_email_as_admin(&dto.email, &state.pool).await {
        Ok(user) => {
            let envy = state.envy.clone();

            tokio::spawn(async move {
                let access_token =
                    sign_jwt(&user.id, &envy.jwt_secret, Some(PepperType::EDIT_PASSWORD));
                let template =
                    request_password_update_template(&user, &access_token, &envy.frontend_url);
                mail::service::send_mail(&user.email, &template.0, &template.1, &envy).await
            });

            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub async fn process_password_edit(
    access_token: &str,
    dto: &EditPasswordDto,
    state: &AppState,
) -> Result<(), ApiError> {
    match decode_jwt(
        access_token.to_string(),
        &state.envy.jwt_secret,
        Some(PepperType::EDIT_PASSWORD),
    ) {
        Ok(claims) => {
            users::service::edit_user_password_by_id_as_admin(&claims.id, dto, &state.pool).await
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

pub async fn refresh(dto: &RefreshDeviceDto, state: &AppState) -> Result<AccessInfo, ApiError> {
    match devices::service::refresh_device_as_admin(dto, &state.pool).await {
        Ok(_) => Ok(AccessInfo {
            access_token: sign_jwt(&dto.user_id, &state.envy.jwt_secret, None),
            refresh_token: None,
            device_id: None,
        }),
        Err(e) => Err(e),
    }
}

pub async fn logout(dto: &LogoutDeviceDto, claims: &Claims, pool: &PgPool) -> Result<(), ApiError> {
    match devices::service::logout_devices_with_ids(&dto.device_ids, claims, pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn delete_account(
    dto: &DeleteAccountDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<(), ApiError> {
    match users::service::get_user_by_id_as_admin(&claims.id, pool).await {
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

            match users::service::set_user_delete_pending_by_id_as_admin(&claims.id, pool).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}
