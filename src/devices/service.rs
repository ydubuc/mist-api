use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    app::{
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::{
            sqlx::{get_code_from_db_err, SqlStateCodes},
            time,
        },
    },
    auth::jwt::models::claims::Claims,
    users::models::user::User,
};

use super::{
    dtos::{
        edit_device_dto::EditDeviceDto, get_devices_filter_dto::GetDevicesFilterDto,
        logout_device_dto::LogoutDeviceDto, refresh_device_dto::RefreshDeviceDto,
    },
    errors::DevicesApiError,
    models::device::Device,
};

pub async fn create_device_as_admin(user: &User, pool: &PgPool) -> Result<Device, ApiError> {
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

    match sqlx_result {
        Ok(_) => Ok(device),
        Err(e) => {
            let Some(db_err) = e.as_database_error()
            else {
                tracing::error!(%e);
                return Err(DefaultApiError::InternalServerError.value());
            };

            let Some(code) = get_code_from_db_err(db_err)
            else {
                tracing::error!(%e);
                return Err(DefaultApiError::InternalServerError.value());
            };

            match code.as_str() {
                SqlStateCodes::UNIQUE_VIOLATION => Err(ApiError {
                    code: StatusCode::CONFLICT,
                    message: "Device already exists.".to_string(),
                }),
                _ => {
                    tracing::error!(%e);
                    Err(DefaultApiError::InternalServerError.value())
                }
            }
        }
    }
}

pub async fn get_devices(
    dto: &GetDevicesFilterDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<Vec<Device>, ApiError> {
    if dto.user_id != claims.id {
        return Err(ApiError {
            code: StatusCode::UNAUTHORIZED,
            message: "Permission denied.".to_string(),
        });
    }

    return get_devices_as_admin(dto, pool).await;
}

pub async fn get_devices_as_admin(
    dto: &GetDevicesFilterDto,
    pool: &PgPool,
) -> Result<Vec<Device>, ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result
    else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, Device>(&sql);

    if let Some(id) = &dto.id {
        sqlx = sqlx.bind(id);
    }
    sqlx = sqlx.bind(&dto.user_id);

    match sqlx.fetch_all(pool).await {
        Ok(devices) => Ok(devices),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn refresh_device_as_admin(
    dto: &RefreshDeviceDto,
    pool: &PgPool,
) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        UPDATE devices SET updated_at = $1
        WHERE id = $2 AND user_id = $3 AND refresh_token = $4
        ",
    )
    .bind(time::current_time_in_secs() as i64)
    .bind(&dto.device_id)
    .bind(&dto.user_id)
    .bind(&dto.refresh_token)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(ApiError {
                code: StatusCode::NOT_FOUND,
                message: "Failed to refresh.".to_string(),
            }),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn edit_device_by_id(
    id: &str,
    dto: &EditDeviceDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<Device, ApiError> {
    println!("{:?}", dto);

    let sql_result = dto.to_sql(claims);
    let Ok(sql) = sql_result
    else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, Device>(&sql);

    if let Some(messaging_token) = &dto.messaging_token {
        sqlx = sqlx.bind(messaging_token);
    }
    sqlx = sqlx.bind(id);

    match sqlx.fetch_optional(pool).await {
        Ok(device) => match device {
            Some(device) => Ok(device),
            None => Err(DevicesApiError::DeviceNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn logout_device_as_admin(dto: &LogoutDeviceDto, pool: &PgPool) -> Result<(), ApiError> {
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

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(DevicesApiError::DeviceNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn delete_devices_with_messaging_tokens_as_admin(
    messaging_tokens: Vec<String>,
    pool: &PgPool,
) -> Result<(), ApiError> {
    let mut sql = "DELETE FROM devices WHERE messaging_token IN (".to_string();

    for i in 0..messaging_tokens.len() {
        if i != messaging_tokens.len() - 1 {
            sql.push_str(&[&messaging_tokens[i], ", "].concat());
        } else {
            sql.push_str(&messaging_tokens[i]);
        }
    }

    sql.push_str(")");

    println!("{}", sql);

    let sqlx = sqlx::query(&sql);

    match sqlx.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}
