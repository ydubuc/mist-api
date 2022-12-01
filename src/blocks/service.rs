use sqlx::PgPool;

use crate::{
    app::{
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::sqlx::{get_code_from_db_err, SqlStateCodes},
    },
    auth::jwt::models::claims::Claims,
};

use super::{dtos::get_blocks_filter_dto::GetBlocksFilterDto, models::block::Block};

pub async fn block(id: &str, claims: &Claims, pool: &PgPool) -> Result<(), ApiError> {
    let block = Block::new(claims, id);

    let sqlx_result = sqlx::query(
        "
        INSERT INTO blocks (
            id, user_id, blocked_id, blocked_at
        )
        VALUES ($1, $2, $3, $4)
        ",
    )
    .bind(block.id)
    .bind(block.user_id)
    .bind(block.blocked_id)
    .bind(block.blocked_at)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(_) => Ok(()),
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
                SqlStateCodes::UNIQUE_VIOLATION => Ok(()),
                _ => {
                    tracing::error!(%e);
                    Err(DefaultApiError::InternalServerError.value())
                }
            }
        }
    }
}

pub async fn get_blocks(
    dto: &GetBlocksFilterDto,
    _claims: &Claims,
    pool: &PgPool,
) -> Result<Vec<Block>, ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result
    else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, Block>(&sql);

    if let Some(id) = &dto.id {
        sqlx = sqlx.bind(id);
    }
    if let Some(user_id) = &dto.user_id {
        sqlx = sqlx.bind(user_id);
    }
    if let Some(blocked_id) = &dto.blocked_id {
        sqlx = sqlx.bind(blocked_id);
    }

    match sqlx.fetch_all(pool).await {
        Ok(blocks) => Ok(blocks),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn unblock(id: &str, claims: &Claims, pool: &PgPool) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        DELETE FROM blocks
        WHERE id = $1
        ",
    )
    .bind(format!("{}{}", claims.id, id))
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}
