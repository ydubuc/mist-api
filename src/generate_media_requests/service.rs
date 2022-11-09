use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    app::{
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::sqlx::{get_code_from_db_err, SqlStateCodes},
    },
    auth::jwt::models::claims::Claims,
    media::dtos::generate_media_dto::GenerateMediaDto,
};

use super::{
    enums::generate_media_request_status::GenerateMediaRequestStatus,
    models::generate_media_request::GenerateMediaRequest,
};

pub async fn create_generate_media_request(
    dto: &GenerateMediaDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<GenerateMediaRequest, ApiError> {
    let generate_media_request = GenerateMediaRequest::new(claims, dto);

    let sqlx_result = sqlx::query(
        "
        INSERT INTO generate_media_requests (
            id, status, generate_media_dto, created_at
        )
        VALUES ($1, $2, $3, $4)
        ",
    )
    .bind(&generate_media_request.id)
    .bind(&generate_media_request.status)
    .bind(&generate_media_request.generate_media_dto)
    .bind(&generate_media_request.created_at)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(_) => Ok(generate_media_request),
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
                    message: "Request already exists.".to_string(),
                }),
                _ => {
                    tracing::error!(%e);
                    Err(DefaultApiError::InternalServerError.value())
                }
            }
        }
    }
}

pub async fn edit_generate_media_request_by_id(
    id: &str,
    status: GenerateMediaRequestStatus,
    pool: &PgPool,
) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        UPDATE generate_media_requests SET status = $1
        WHERE id = $2
        ",
    )
    .bind(status.value())
    .bind(id)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(ApiError {
                code: StatusCode::NOT_FOUND,
                message: "Failed to update.".to_string(),
            }),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}
