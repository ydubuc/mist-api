use axum::extract::Multipart;
use b2_backblaze::B2;
use sqlx::PgPool;

use crate::{
    app::{errors::DefaultApiError, models::api_error::ApiError},
    auth::jwt::models::claims::Claims,
};

use super::{
    backblaze,
    dtos::{generate_media_dto::GenerateMediaDto, get_media_filter_dto::GetMediaFilterDto},
    errors::MediaApiError,
    models::{import_media_response::ImportMediaResponse, media::Media},
    services::dalle,
};

pub async fn generate_media(
    dto: &GenerateMediaDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<Vec<Media>, ApiError> {
    match dalle::service::create_media(claims, dto).await {
        Ok(media) => upload_media(media, pool).await,
        Err(e) => Err(e),
    }
}

pub async fn import_media(
    multipart: Multipart,
    claims: &Claims,
    pool: &PgPool,
    b2: &B2,
) -> Result<Vec<Media>, ApiError> {
    match backblaze::service::upload_files(multipart, b2).await {
        Ok(responses) => {
            let mut vec = Vec::new();

            for res in responses {
                let download_url = [
                    &b2.downloadUrl,
                    "/b2api/v1/b2_download_file_by_id?fileId=",
                    &res.file_id,
                ]
                .concat();

                let import_media_res = ImportMediaResponse {
                    id: res.file_id.to_string(),
                    download_url,
                    backblaze_upload_file_response: res,
                };

                vec.push(Media::from_import(&import_media_res, claims));
            }

            return upload_media(vec, pool).await;
        }
        Err(e) => Err(e),
    }
}

async fn upload_media(media: Vec<Media>, pool: &PgPool) -> Result<Vec<Media>, ApiError> {
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

    match sqlx.execute(pool).await {
        Ok(_) => Ok(media),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn get_media(
    dto: &GetMediaFilterDto,
    _claims: &Claims,
    pool: &PgPool,
) -> Result<Vec<Media>, ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result
    else {
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

    match sqlx.fetch_all(pool).await {
        Ok(media) => Ok(media),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn get_media_by_id(id: &str, _claims: &Claims, pool: &PgPool) -> Result<Media, ApiError> {
    let sqlx_result = sqlx::query_as::<_, Media>(
        "
        SELECT * FROM media WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match sqlx_result {
        Ok(post) => match post {
            Some(post) => Ok(post),
            None => Err(MediaApiError::MediaNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn delete_media_by_id(id: &str, claims: &Claims, pool: &PgPool) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        DELETE FROM media WHERE id = $1 AND user_id = $2
        ",
    )
    .bind(id)
    .bind(&claims.id)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(MediaApiError::MediaNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}
