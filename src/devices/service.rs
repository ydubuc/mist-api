use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    app::{
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::sqlx::{get_code_from_db_err, SqlStateCodes},
    },
    users::models::user::User,
};

use super::{
    dtos::{logout_device_dto::LogoutDeviceDto, refresh_device_dto::RefreshDeviceDto},
    errors::DevicesApiError,
    models::device::Device,
};

pub async fn create_device(user: &User, pool: &PgPool) -> Result<Device, ApiError> {
    let device = Device::new(user);

    let sqlx_result = sqlx::query(
        "
        INSERT INTO devices (
            id, user_id, refresh_token, updated_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5)
        ",
    )
    .bind(&device.id)
    .bind(&device.user_id)
    .bind(&device.refresh_token)
    .bind(device.updated_at.to_owned() as i64)
    .bind(device.created_at.to_owned() as i64)
    .execute(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(_) => Ok(device),
        Err(e) => {
            let Some(db_err) = e.as_database_error() else {
                return Err(DefaultApiError::InternalServerError.value());
            };

            let Some(code) = get_code_from_db_err(db_err) else {
                return Err(DefaultApiError::InternalServerError.value());
            };

            match code.as_str() {
                SqlStateCodes::UNIQUE_VIOLATION => Err(ApiError {
                    code: StatusCode::CONFLICT,
                    message: "Device already exists.".to_string(),
                }),
                _ => Err(DefaultApiError::InternalServerError.value()),
            }
        }
    }
}

pub async fn refresh_device(dto: &RefreshDeviceDto, pool: &PgPool) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        UPDATE devices SET updated_at = $1
        WHERE id = $2 AND user_id = $3 AND refresh_token = $4
        ",
    )
    .bind(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    )
    .bind(&dto.device_id)
    .bind(&dto.user_id)
    .bind(&dto.refresh_token)
    .execute(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(ApiError {
                code: StatusCode::NOT_FOUND,
                message: "Failed to refresh.".to_string(),
            }),
        },
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}

pub async fn logout_device(dto: &LogoutDeviceDto, pool: &PgPool) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        DELETE FROM devices
        WHERE id = $1 AND user_id = $2 AND refresh_token = $3
        ",
    )
    .bind(&dto.device_id)
    .bind(&dto.user_id)
    .bind(&dto.refresh_token)
    .execute(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(DevicesApiError::DeviceNotFound.value()),
        },
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}
