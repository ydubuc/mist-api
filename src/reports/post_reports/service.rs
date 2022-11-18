use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    app::{
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::sqlx::{get_code_from_db_err, SqlStateCodes},
    },
    auth::jwt::models::claims::Claims,
};

pub async fn report_post_by_id(id: &str, claims: &Claims, pool: &PgPool) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        INSERT INTO post_reports (
            id, post_id, user_id
        )
        VALUES ($1, $2, $3)
        ",
    )
    .bind(Uuid::new_v4().to_string())
    .bind(id)
    .bind(&claims.id)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(_) => {
            let _ = sqlx::query(
                "
                UPDATE posts SET reports_count = reports_count + 1
                WHERE id = $1
                ",
            )
            .bind(id)
            .execute(pool)
            .await;

            Ok(())
        }
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
                    message: "You have already reported this.".to_string(),
                }),
                _ => {
                    tracing::error!(%e);
                    Err(DefaultApiError::InternalServerError.value())
                }
            }
        }
    }
}
