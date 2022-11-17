use reqwest::StatusCode;
use sqlx::Postgres;

use crate::app::models::api_error::ApiError;

pub async fn update_user_ink_by_id(
    id: &str,
    amount: i32,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<(), ApiError> {
    println!("update_user_ink_by_id");

    let sql = match amount >= 0 {
        true => {
            r#"
            UPDATE users
            SET ink = ink + $1
            WHERE id = $2
            "#
        }
        false => {
            r#"
            UPDATE users
            SET ink = ink - $1
            WHERE id = $2
            "#
        }
    };

    match sqlx::query(&sql)
        .bind(amount)
        .bind(id)
        .execute(&mut *tx)
        .await
    {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => {
                tracing::error!("WEBHOOK ERROR<update_user_ink_by_id>: NO ROWS AFFECTED");
                tracing::error!(%id);

                return Err(ApiError {
                    code: StatusCode::NOT_FOUND,
                    message: "User not found.".to_string(),
                });
            }
        },
        Err(e) => {
            tracing::error!(%e);
            return Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to update user ink.".to_string(),
            });
        }
    }
}
