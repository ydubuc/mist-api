use sqlx::PgPool;

use crate::{
    app::{errors::DefaultApiError, models::api_error::ApiError},
    auth::jwt::models::claims::Claims,
};

use super::{
    dtos::{create_media_dto::CreateMediaDto, get_media_filter_dto::GetMediaFilterDto},
    errors::MediaApiError,
    models::media::Media,
    services::dalle,
};

pub async fn create_media(
    claims: &Claims,
    dto: &CreateMediaDto,
    pool: &PgPool,
) -> Result<Vec<Media>, ApiError> {
    match dalle::create_media(claims, dto).await {
        Ok(media) => {
            println!("{:?}", media);

            let mut sql = "
            INSERT INTO media (
                id, user_id, url, width, height, mime_type, source, created_at
            ) "
            .to_string();

            let mut index: u8 = 1;
            for i in 0..media.len() {
                if i == 0 {
                    sql.push_str("VALUES (");
                } else {
                    sql.push_str(", (");
                }

                for j in 0..8 {
                    sql.push_str(&["$", &index.to_string()].concat());
                    index += 1;

                    if j != 7 {
                        sql.push_str(", ");
                    }
                }

                sql.push_str(")");
            }

            println!("{}", sql);

            let mut sqlx = sqlx::query(&sql);

            for m in &media {
                sqlx = sqlx.bind(&m.id);
                sqlx = sqlx.bind(&m.user_id);
                sqlx = sqlx.bind(&m.url);
                sqlx = sqlx.bind(m.width.to_owned() as i16);
                sqlx = sqlx.bind(m.height.to_owned() as i16);
                sqlx = sqlx.bind(&m.mime_type);
                sqlx = sqlx.bind(&m.source);
                sqlx = sqlx.bind(m.created_at.to_owned() as i64);
            }

            let sqlx_result = sqlx.execute(pool).await;

            if let Some(error) = sqlx_result.as_ref().err() {
                println!("{}", error);
            }

            match sqlx_result {
                Ok(_) => Ok(media),
                Err(_) => Err(DefaultApiError::InternalServerError.value()),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn get_media(
    _claims: &Claims,
    dto: &GetMediaFilterDto,
    pool: &PgPool,
) -> Result<Vec<Media>, ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, Media>(&sql);

    if let Some(id) = &dto.id {
        sqlx = sqlx.bind(id);
    }
    if let Some(user_id) = &dto.user_id {
        sqlx = sqlx.bind(user_id);
    }
    if let Some(url) = &dto.url {
        sqlx = sqlx.bind(url);
    }
    if let Some(mime_type) = &dto.mime_type {
        sqlx = sqlx.bind(mime_type);
    }

    let sqlx_result = sqlx.fetch_all(pool).await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(media) => Ok(media),
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}

pub async fn get_media_by_id(id: &str, pool: &PgPool) -> Result<Media, ApiError> {
    let sqlx_result = sqlx::query_as::<_, Media>(
        "
        SELECT * FROM media WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(post) => match post {
            Some(post) => Ok(post),
            None => Err(MediaApiError::MediaNotFound.value()),
        },
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}

pub async fn delete_media_by_id(claims: &Claims, id: &str, pool: &PgPool) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        DELETE FROM media WHERE id = $1 AND user_id = $2
        ",
    )
    .bind(id)
    .bind(&claims.id)
    .execute(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(MediaApiError::MediaNotFound.value()),
        },
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}
