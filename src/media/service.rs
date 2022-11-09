use axum::{extract::Multipart, http::StatusCode};
use b2_backblaze::B2;
use imagesize::ImageSize;
use sqlx::PgPool;

use crate::{
    app::{
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::{
            multipart::{models::file_properties::FileProperties, multipart::get_files_properties},
            time,
        },
    },
    auth::jwt::models::claims::Claims,
    posts, users,
};

use super::{
    dtos::{generate_media_dto::GenerateMediaDto, get_media_filter_dto::GetMediaFilterDto},
    enums::{media_generator::MediaGenerator, media_source::MediaSource},
    errors::MediaApiError,
    models::media::Media,
    util::{
        backblaze::{self, models::backblaze_upload_file_response::BackblazeUploadFileResponse},
        dalle,
    },
};

pub async fn generate_media(
    dto: &GenerateMediaDto,
    claims: &Claims,
    pool: &PgPool,
    b2: &B2,
) -> Result<Vec<Media>, ApiError> {
    match dto.generator.as_ref() {
        MediaGenerator::DALLE => {
            match dalle::service::generate_media(dto, claims, pool, b2).await {
                Ok(media) => {
                    users::service::send_notifications_to_user_id_as_admin(
                        "Mist",
                        "Your images are ready!",
                        &claims.id,
                        pool,
                    )
                    .await;

                    match posts::service::create_post_with_media(dto, &media, claims, pool).await {
                        Ok(_) => Ok(media),
                        Err(e) => Err(e),
                    }
                }
                Err(e) => Err(e),
            }
        }
        _ => Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: "Media generator not supported.".to_string(),
        }),
    }
}

pub async fn import_media(
    multipart: Multipart,
    claims: &Claims,
    pool: &PgPool,
    b2: &B2,
) -> Result<Vec<Media>, ApiError> {
    let files_properties = get_files_properties(multipart).await;

    if files_properties.len() == 0 {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: "Received nothing to upload.".to_string(),
        });
    }

    for file_properties in &files_properties {
        if file_properties.mime_type.type_() != mime::IMAGE {
            return Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: "Files must be of type image.".to_string(),
            });
        }
    }

    for file_properties in &files_properties {
        let Ok(size) = imagesize::blob_size(&file_properties.data)
        else {
            return Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to get image size.".to_string()
            })
        };
    }

    let sub_folder = Some(["media/", &claims.id].concat());

    match backblaze::service::upload_files(files_properties, &sub_folder, b2).await {
        Ok(responses) => {
            let media = create_media_from_responses(responses, MediaSource::Import, claims, b2);

            if media.len() == 0 {
                return Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "Failed to upload files.".to_string(),
                });
            }

            return upload_media(media, pool).await;
        }
        Err(e) => Err(e),
    }
}

pub fn create_media_from_responses(
    responses: Vec<(FileProperties, BackblazeUploadFileResponse)>,
    source: MediaSource,
    claims: &Claims,
    b2: &B2,
) -> Vec<Media> {
    let mut vec = Vec::new();

    for res in responses {
        let download_url = [
            &b2.downloadUrl,
            "/b2api/v1/b2_download_file_by_id?fileId=",
            &res.1.file_id,
        ]
        .concat();

        let size = match imagesize::blob_size(&res.0.data) {
            Ok(size) => size,
            Err(e) => ImageSize {
                width: 512,
                height: 512,
            },
        };

        let media = Media {
            id: res.0.id.to_string(),
            user_id: claims.id.to_string(),
            file_id: res.1.file_id.to_string(),
            url: download_url,
            width: size.width as i16,
            height: size.height as i16,
            mime_type: res.0.mime_type.to_string(),
            source: source.value(),
            created_at: time::current_time_in_secs() as i64,
        };

        vec.push(media);
    }

    return vec;
}

pub async fn upload_media(media: Vec<Media>, pool: &PgPool) -> Result<Vec<Media>, ApiError> {
    let num_properties: u8 = 9;

    let mut sql = "
    INSERT INTO media (
        id, user_id, file_id, url, width, height, mime_type, source, created_at
    ) "
    .to_string();

    let mut index: u8 = 1;
    for i in 0..media.len() {
        if i == 0 {
            sql.push_str("VALUES (");
        } else {
            sql.push_str(", (");
        }

        for j in 0..num_properties {
            sql.push_str(&["$", &index.to_string()].concat());
            index += 1;

            if j != num_properties - 1 {
                sql.push_str(", ");
            }
        }

        sql.push_str(")");
    }

    let mut sqlx = sqlx::query(&sql);

    for m in &media {
        sqlx = sqlx.bind(&m.id);
        sqlx = sqlx.bind(&m.user_id);
        sqlx = sqlx.bind(&m.file_id);
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
    claims: &Claims,
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

pub async fn get_media_by_id(id: &str, claims: &Claims, pool: &PgPool) -> Result<Media, ApiError> {
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

pub async fn delete_media_by_id(
    id: &str,
    claims: &Claims,
    pool: &PgPool,
    b2: &B2,
) -> Result<(), ApiError> {
    match get_media_by_id(id, claims, pool).await {
        Ok(media) => {
            let file_name = ["media/", &claims.id, "/", id].concat();
            match backblaze::service::delete_file(&file_name, &media.file_id, b2).await {
                Ok(_) => {
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
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}
