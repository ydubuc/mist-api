use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    app::{
        self,
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::{
            hasher,
            sqlx::{get_code_from_db_err, SqlStateCodes},
        },
    },
    auth::{
        dtos::{login_dto::LoginDto, register_dto::RegisterDto},
        jwt::models::claims::Claims,
    },
    devices::{self, dtos::get_devices_filter_dto::GetDevicesFilterDto},
};

use super::{
    dtos::{edit_user_dto::EditUserDto, get_users_filter_dto::GetUsersFilterDto},
    errors::UsersApiError,
    models::user::User,
};

pub async fn create_user_as_admin(dto: &RegisterDto, pool: &PgPool) -> Result<User, ApiError> {
    let Ok(hash) = hasher::hash(dto.password.to_string()).await
    else {
        return Err(DefaultApiError::InternalServerError.value());
    };

    let user = User::new(dto, hash);

    let sqlx_result = sqlx::query(
        "
        INSERT INTO users (
            id, username, username_key, displayname,
            email, email_key, password_hash, updated_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ",
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.username_key)
    .bind(&user.displayname)
    .bind(&user.email)
    .bind(&user.email_key)
    .bind(&user.password_hash)
    .bind(user.updated_at.to_owned() as i64)
    .bind(user.created_at.to_owned() as i64)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(_) => Ok(user),
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
                    message: "User already exists.".to_string(),
                }),
                _ => {
                    tracing::error!(%e);
                    Err(DefaultApiError::InternalServerError.value())
                }
            }
        }
    }
}

pub async fn get_users(
    dto: &GetUsersFilterDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<Vec<User>, ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result
    else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, User>(&sql);

    if let Some(id) = &dto.id {
        sqlx = sqlx.bind(id);
    }
    if let Some(username) = &dto.username {
        sqlx = sqlx.bind(["%", &username.to_lowercase(), "%"].concat());
    }

    let sqlx_result = sqlx.fetch_all(pool).await;

    match sqlx_result {
        Ok(users) => Ok(users),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn get_user_by_id(id: &str, claims: &Claims, pool: &PgPool) -> Result<User, ApiError> {
    let sqlx_result = sqlx::query_as::<_, User>(
        "
        SELECT * FROM users WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match sqlx_result {
        Ok(user) => match user {
            Some(user) => Ok(user),
            None => Err(UsersApiError::UserNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(UsersApiError::UserNotFound.value())
        }
    }
}

pub async fn get_user_by_login_dto_as_admin(
    login_dto: &LoginDto,
    pool: &PgPool,
) -> Result<User, ApiError> {
    if let Some(username) = &login_dto.username {
        return get_user_by_username_as_admin(username, pool).await;
    }
    if let Some(email) = &login_dto.email {
        return get_user_by_email_as_admin(email, pool).await;
    }

    Err(ApiError {
        code: StatusCode::BAD_REQUEST,
        message: "Missing credentials.".to_string(),
    })
}

pub async fn get_user_by_username_as_admin(
    username: &str,
    pool: &PgPool,
) -> Result<User, ApiError> {
    let sqlx_result = sqlx::query_as::<_, User>(
        "
        SELECT * FROM users
        WHERE username_key = $1
        ",
    )
    .bind(username.to_lowercase())
    .fetch_optional(pool)
    .await;

    match sqlx_result {
        Ok(user) => match user {
            Some(user) => Ok(user),
            None => Err(UsersApiError::UserNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(UsersApiError::UserNotFound.value())
        }
    }
}

pub async fn get_user_by_email_as_admin(email: &str, pool: &PgPool) -> Result<User, ApiError> {
    let sqlx_result = sqlx::query_as::<_, User>(
        "
        SELECT * FROM users
        WHERE email_key = $1
        ",
    )
    .bind(email.to_lowercase())
    .fetch_optional(pool)
    .await;

    match sqlx_result {
        Ok(user) => match user {
            Some(user) => Ok(user),
            None => Err(UsersApiError::UserNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(UsersApiError::UserNotFound.value())
        }
    }
}

pub async fn edit_user_by_id(
    id: &str,
    dto: &EditUserDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<User, ApiError> {
    if claims.id != id {
        return Err(UsersApiError::PermissionDenied.value());
    }

    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result
    else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, User>(&sql);

    if let Some(username) = &dto.username {
        sqlx = sqlx.bind(username);
        sqlx = sqlx.bind(username.to_lowercase());
    }
    if let Some(displayname) = &dto.displayname {
        sqlx = sqlx.bind(displayname);
    }
    sqlx = sqlx.bind(id);

    match sqlx.fetch_optional(pool).await {
        Ok(user) => match user {
            Some(user) => Ok(user),
            None => Err(UsersApiError::UserNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn delete_user_by_id(id: &str, claims: &Claims, pool: &PgPool) -> Result<(), ApiError> {
    if claims.id != id {
        return Err(UsersApiError::PermissionDenied.value());
    }

    let sqlx_result = sqlx::query(
        "
        DELETE FROM users WHERE id = $1
        ",
    )
    .bind(id)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(UsersApiError::UserNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn send_notifications_to_user_id_as_admin(
    title: &str,
    body: &str,
    id: &str,
    pool: &PgPool,
) {
    let dto = GetDevicesFilterDto {
        id: None,
        user_id: id.to_string(),
        sort: None,
        cursor: None,
        limit: None,
    };

    match devices::service::get_devices_as_admin(&dto, pool).await {
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
                let _ = devices::service::delete_devices_with_messaging_tokens_as_admin(
                    failed_messaging_tokens,
                    pool,
                )
                .await;
            }
        }
        Err(e) => {}
    }
}
