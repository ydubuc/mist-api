use sqlx::PgPool;

use crate::{
    app::{
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::sqlx::{get_code_from_db_err, SqlStateCodes},
    },
    auth::jwt::models::claims::Claims,
};

use super::{dtos::get_follows_filter_dto::GetFollowsFilterDto, models::follow::Follow};

pub async fn follow(id: &str, claims: &Claims, pool: &PgPool) -> Result<(), ApiError> {
    let follow = Follow::new(claims, id);

    let sqlx_result = sqlx::query(
        "
        INSERT INTO follows (
            id, user_id, follows_id, followed_at
        )
        VALUES ($1, $2, $3, $4)
        ",
    )
    .bind(follow.id)
    .bind(follow.user_id)
    .bind(follow.follows_id)
    .bind(follow.followed_at)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(_) => Ok(()),
        Err(e) => {
            let Some(db_err) = e.as_database_error() else {
                tracing::error!(%e);
                return Err(DefaultApiError::InternalServerError.value());
            };

            let Some(code) = get_code_from_db_err(db_err) else {
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

pub async fn get_follows(
    dto: &GetFollowsFilterDto,
    _claims: &Claims,
    pool: &PgPool,
) -> Result<Vec<Follow>, ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, Follow>(&sql);

    if let Some(id) = &dto.id {
        sqlx = sqlx.bind(id);
    }
    if let Some(user_id) = &dto.user_id {
        sqlx = sqlx.bind(user_id);
    }
    if let Some(follows_id) = &dto.follows_id {
        sqlx = sqlx.bind(follows_id);
    }

    match sqlx.fetch_all(pool).await {
        Ok(follows) => Ok(follows),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn unfollow(id: &str, claims: &Claims, pool: &PgPool) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        DELETE FROM follows
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
