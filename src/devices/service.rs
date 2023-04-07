use std::sync::Arc;

use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    app::{
        self,
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::{
            sqlx::{get_code_from_db_err, SqlStateCodes},
            time,
        },
    },
    auth::jwt::models::claims::Claims,
    users::models::user::User,
    AppState,
};

use super::{
    dtos::{
        edit_device_dto::EditDeviceDto, get_devices_filter_dto::GetDevicesFilterDto,
        refresh_device_dto::RefreshDeviceDto,
    },
    errors::DevicesApiError,
    models::device::Device,
};

pub async fn create_device_as_admin(user: &User, pool: &PgPool) -> Result<Device, ApiError> {
    let device = Device::new(user);

    let sqlx_result = sqlx::query(
        "
        INSERT INTO devices (
            id, user_id, refresh_token, roles, updated_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
    )
    .bind(&device.id)
    .bind(&device.user_id)
    .bind(&device.refresh_token)
    .bind(&device.roles)
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

pub fn send_notifications_to_devices_with_user_id(
    title: String,
    body: String,
    click_action: Option<String>,
    id: String,
    state: Arc<AppState>,
) {
    let dto = GetDevicesFilterDto {
        id: None,
        user_id: id.to_string(),
        sort: None,
        cursor: None,
        limit: None,
    };

    tokio::spawn(async move {
        match get_devices_as_admin(&dto, &state.pool).await {
            Ok(devices) => {
                let mut futures = Vec::new();

                for device in devices {
                    let Some(messaging_token) = device.messaging_token
                    else {
                        continue;
                    };

                    futures.push(app::util::fcm::send_notification(
                        messaging_token.to_string(),
                        title.to_string(),
                        body.to_string(),
                        click_action.clone(),
                        state.envy.fcm_api_key.to_string(),
                        state.fcm_client.clone(),
                    ));
                }

                let results = futures::future::join_all(futures).await;
                let mut failed_messaging_tokens = Vec::new();

                for result in results {
                    if result.is_err() {
                        failed_messaging_tokens.push(result.unwrap_err())
                    }
                }

                if failed_messaging_tokens.len() > 0 {
                    let _ = delete_devices_with_messaging_tokens_as_admin(
                        failed_messaging_tokens,
                        &state.pool,
                    )
                    .await;
                }
            }
            Err(_) => {
                // quietly fail :(
            }
        }
    });
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
) -> Result<Device, ApiError> {
    let sqlx_result = sqlx::query_as::<_, Device>(
        "
        UPDATE devices SET updated_at = $1
        WHERE id = $2 AND user_id = $3 AND refresh_token = $4
        RETURNING *;
        ",
    )
    .bind(time::current_time_in_secs() as i64)
    .bind(&dto.device_id)
    .bind(&dto.user_id)
    .bind(&dto.refresh_token)
    .fetch_optional(pool)
    .await;

    match sqlx_result {
        Ok(device) => match device {
            Some(dev) => Ok(dev),
            None => Err(ApiError {
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

pub async fn logout_devices_with_ids(
    ids: &Vec<String>,
    claims: &Claims,
    pool: &PgPool,
) -> Result<(), ApiError> {
    let mut sql = "DELETE FROM devices WHERE id IN (".to_string();

    for i in 0..ids.len() {
        if i != ids.len() - 1 {
            sql.push_str(&["'", &ids[i], "', "].concat());
        } else {
            sql.push_str(&["'", &ids[i], "'"].concat());
        }
    }

    sql.push_str(")");
    sql.push_str(" AND user_id = $1");

    let sqlx = sqlx::query(&sql).bind(&claims.id);

    match sqlx.execute(pool).await {
        Ok(_) => Ok(()),
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
            sql.push_str(&["'", &messaging_tokens[i], "', "].concat());
        } else {
            sql.push_str(&["'", &messaging_tokens[i], "'"].concat());
        }
    }

    sql.push_str(")");

    let sqlx = sqlx::query(&sql);

    match sqlx.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}
