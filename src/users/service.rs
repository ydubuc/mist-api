use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    app::{
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
};

use super::{
    dtos::{edit_user_dto::EditUserDto, get_users_filter_dto::GetUsersFilterDto},
    errors::UsersApiError,
    models::user::User,
};

pub async fn create_user(dto: &RegisterDto, pool: &PgPool) -> Result<User, ApiError> {
    let Ok(hash) = hasher::hash(dto.password.to_string()).await else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to hash password.".to_string()
        });
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

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(_) => Ok(user),
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
                    message: "User already exists.".to_string(),
                }),
                _ => Err(DefaultApiError::InternalServerError.value()),
            }
        }
    }
}

pub async fn get_users(dto: &GetUsersFilterDto, pool: &PgPool) -> Result<Vec<User>, ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result else {
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

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(users) => Ok(users),
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}

pub async fn get_user_by_id(id: &str, pool: &PgPool) -> Result<User, ApiError> {
    let sqlx_result = sqlx::query_as::<_, User>(
        "
        SELECT * FROM users WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(user) => match user {
            Some(user) => Ok(user),
            None => Err(UsersApiError::UserNotFound.value()),
        },
        Err(_) => Err(UsersApiError::UserNotFound.value()),
    }
}

pub async fn get_user_by_login_dto(login_dto: &LoginDto, pool: &PgPool) -> Result<User, ApiError> {
    if let Some(username) = &login_dto.username {
        return get_user_by_username(username, pool).await;
    }
    if let Some(email) = &login_dto.email {
        return get_user_by_email(email, pool).await;
    }

    Err(ApiError {
        code: StatusCode::BAD_REQUEST,
        message: "Missing credentials.".to_string(),
    })
}

pub async fn get_user_by_username(username: &str, pool: &PgPool) -> Result<User, ApiError> {
    let sqlx_result = sqlx::query_as::<_, User>(
        "
        SELECT * FROM users
        WHERE username_key = $1
        ",
    )
    .bind(username.to_lowercase())
    .fetch_optional(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(user) => match user {
            Some(user) => Ok(user),
            None => Err(UsersApiError::UserNotFound.value()),
        },
        Err(_) => Err(UsersApiError::UserNotFound.value()),
    }
}

pub async fn get_user_by_email(email: &str, pool: &PgPool) -> Result<User, ApiError> {
    let sqlx_result = sqlx::query_as::<_, User>(
        "
        SELECT * FROM users
        WHERE email_key = $1
        ",
    )
    .bind(email.to_lowercase())
    .fetch_optional(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(user) => match user {
            Some(user) => Ok(user),
            None => Err(UsersApiError::UserNotFound.value()),
        },
        Err(_) => Err(UsersApiError::UserNotFound.value()),
    }
}

pub async fn edit_user_by_id(
    claims: &Claims,
    id: &str,
    dto: &EditUserDto,
    pool: &PgPool,
) -> Result<User, ApiError> {
    if claims.id != id {
        return Err(UsersApiError::PermissionDenied.value());
    }

    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, User>(&sql);

    if let Some(displayname) = &dto.displayname {
        sqlx = sqlx.bind(displayname);
    }
    sqlx = sqlx.bind(id);

    let sqlx_result = sqlx.fetch_optional(pool).await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(user) => match user {
            Some(user) => Ok(user),
            None => Err(UsersApiError::UserNotFound.value()),
        },
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}

pub async fn delete_user_by_id(claims: &Claims, id: &str, pool: &PgPool) -> Result<(), ApiError> {
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

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(UsersApiError::UserNotFound.value()),
        },
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}
