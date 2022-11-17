use axum::{extract::Multipart, http::StatusCode};
use b2_backblaze::B2;
use sqlx::PgPool;

use crate::{
    app::{
        errors::DefaultApiError, models::api_error::ApiError,
        util::multipart::multipart::get_files_properties,
    },
    auth::jwt::models::claims::Claims,
    devices,
    generate_media_requests::{
        self, enums::generate_media_request_status::GenerateMediaRequestStatus,
        models::generate_media_request::GenerateMediaRequest,
    },
    posts, transactions, users, AppState,
};

use super::{
    dtos::{generate_media_dto::GenerateMediaDto, get_media_filter_dto::GetMediaFilterDto},
    enums::media_generator::MediaGenerator,
    errors::MediaApiError,
    models::media::Media,
    util::{backblaze, dalle, dream, stable_horde},
};

const SUPPORTED_GENERATORS: [&str; 3] = [
    MediaGenerator::DALLE,
    MediaGenerator::DREAM,
    MediaGenerator::STABLE_HORDE,
];

pub async fn generate_media(
    dto: &GenerateMediaDto,
    claims: &Claims,
    state: &AppState,
) -> Result<GenerateMediaRequest, ApiError> {
    if !SUPPORTED_GENERATORS.contains(&dto.generator.as_ref()) {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: "Media generator not supported.".to_string(),
        });
    }

    if let Err(e) = check_user_ink_and_set_pending(dto, claims, &state.pool).await {
        return Err(e);
    }

    match generate_media_requests::service::create_request(dto, claims, &state.pool).await {
        Ok(generate_media_request) => {
            match dto.generator.as_ref() {
                MediaGenerator::DALLE => {
                    dalle::service::spawn_generate_media_task(
                        generate_media_request.clone(),
                        claims.clone(),
                        state.clone(),
                    );
                }
                MediaGenerator::DREAM => {
                    dream::service::spawn_generate_media_task(
                        generate_media_request.clone(),
                        claims.clone(),
                        state.clone(),
                    );
                }
                MediaGenerator::STABLE_HORDE => {
                    stable_horde::service::spawn_generate_media_task(
                        generate_media_request.clone(),
                        claims.clone(),
                        state.clone(),
                    );
                }
                _ => {
                    // this should not happen
                    return Err(ApiError {
                        code: StatusCode::BAD_REQUEST,
                        message: "Media generator not supported.".to_string(),
                    });
                }
            }

            Ok(generate_media_request)
        }
        Err(e) => Err(e),
    }
}

async fn check_user_ink_and_set_pending(
    dto: &GenerateMediaDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<(), ApiError> {
    let required_ink = calculate_ink(dto);

    let user = match users::service::get_user_by_id_as_admin(&claims.id, pool).await {
        Ok(user) => user,
        Err(e) => return Err(e),
    };

    if user.ink < required_ink {
        return Err(ApiError {
            code: StatusCode::NOT_ACCEPTABLE,
            message: "Insufficient ink.".to_string(),
        });
    }

    // TODO: i need to set a ink pending value on user to prevent multiple requests being accepted

    Ok(())
}

fn calculate_ink(dto: &GenerateMediaDto) -> i64 {
    return 5;
}

pub async fn on_generate_media_completion(
    generate_media_request: &GenerateMediaRequest,
    status: &GenerateMediaRequestStatus,
    media: &Option<Vec<Media>>,
    claims: &Claims,
    state: &AppState,
) -> Result<(), ApiError> {
    let Some(media) = media
    else {
        return generate_media_requests::service::edit_generate_media_request_by_id(
            &generate_media_request.id,
            status,
            &state.pool
        ).await;
    };

    if media.len() < 1 {
        return generate_media_requests::service::edit_generate_media_request_by_id(
            &generate_media_request.id,
            status,
            &state.pool,
        )
        .await;
    }

    if status.value() != GenerateMediaRequestStatus::Completed.value() {
        return generate_media_requests::service::edit_generate_media_request_by_id(
            &generate_media_request.id,
            status,
            &state.pool,
        )
        .await;
    }

    // TODO: retry making tx multiple times
    let Ok(mut tx) = state.pool.begin().await
    else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to begin pool transaction.".to_string(),
        });
    };

    let result_1 = generate_media_requests::service::edit_generate_media_request_by_id_as_tx(
        &generate_media_request.id,
        status,
        &mut tx,
    )
    .await;

    let ink_cost = calculate_ink(&generate_media_request.generate_media_dto);
    let result_2 =
        users::util::ink::update_user_ink_by_id(&claims.id, ink_cost, false, &mut tx).await;

    match tx.commit().await {
        Ok(_) => {
            if result_1.is_err() || result_2.is_err() {
                return Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "Database transaction did not go through.".to_string(),
                });
            }
        }
        Err(e) => {
            tracing::error!(%e);
            return Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Database transaction failed.".to_string(),
            });
        }
    }

    devices::service::send_notifications_to_devices_with_user_id(
        "Mist",
        "Your images are ready!",
        &claims.id,
        state,
    )
    .await;

    match posts::service::create_post_with_media(
        &generate_media_request.generate_media_dto,
        &media,
        &claims,
        &state.pool,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn import_media(
    multipart: Multipart,
    claims: &Claims,
    state: &AppState,
) -> Result<Vec<Media>, ApiError> {
    let files_properties = get_files_properties(multipart).await;

    if files_properties.len() == 0 {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: "Received nothing to upload.".to_string(),
        });
    }

    for file_properties in &files_properties {
        println!("{}", file_properties.mime_type);
        println!("mime from mime {}", mime::IMAGE.to_string());

        if !file_properties
            .mime_type
            .starts_with(&mime::IMAGE.to_string())
        {
            return Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: "Files must be images.".to_string(),
            });
        }
    }

    let mut sizes = Vec::new();

    for file_properties in &files_properties {
        let Ok(size) = imagesize::blob_size(&file_properties.data)
        else {
            return Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to get image size.".to_string()
            })
        };

        sizes.push(size);
    }

    let sub_folder = Some(["media/", &claims.id].concat());

    match backblaze::service::upload_files(&files_properties, &sub_folder, &state.b2).await {
        Ok(responses) => {
            if responses.len() == 0 {
                return Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "Failed to upload files.".to_string(),
                });
            }

            // FIXME: note that if responses are handled in parallel, sizes with not have the right index
            let media = Media::from_import(&responses, &sizes, claims, &state.b2);

            return upload_media(media, &state.pool).await;
        }
        Err(e) => Err(e),
    }
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
        SELECT * FROM media WHERE id = $1 AND user_id = $2
        ",
    )
    .bind(id)
    .bind(&claims.id)
    .fetch_optional(pool)
    .await;

    match sqlx_result {
        Ok(media) => match media {
            Some(media) => Ok(media),
            None => Err(MediaApiError::MediaNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn get_media_by_id_as_admin(id: &str, pool: &PgPool) -> Result<Media, ApiError> {
    let sqlx_result = sqlx::query_as::<_, Media>(
        "
        SELECT * FROM media WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match sqlx_result {
        Ok(media) => match media {
            Some(media) => Ok(media),
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
                    .bind(&media.user_id)
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
