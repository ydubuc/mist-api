use std::sync::Arc;

use axum::{extract::Multipart, http::StatusCode};
use sqlx::{PgPool, Postgres};
use tokio_retry::{strategy::FixedInterval, Retry};

use crate::{
    app::{
        enums::api_status::ApiStatus,
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::multipart::{
            models::file_properties::FileProperties, multipart::get_files_properties,
        },
    },
    auth::jwt::models::claims::Claims,
    devices,
    generate_media_requests::{
        self, enums::generate_media_request_status::GenerateMediaRequestStatus,
        models::generate_media_request::GenerateMediaRequest,
    },
    posts::{self, models::post::Post},
    users::{self, util::ink::dtos::edit_user_ink_dto::EditUserInkDto},
    AppState,
};

use super::{
    apis::{dalle, replicate, stable_horde},
    dtos::{generate_media_dto::GenerateMediaDto, get_media_filter_dto::GetMediaFilterDto},
    enums::{media_generator::MediaGenerator, media_model::MediaModel},
    errors::MediaApiError,
    models::media::Media,
    util::{backblaze, openai},
};

pub async fn generate_media(
    dto: &GenerateMediaDto,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<GenerateMediaRequest, ApiError> {
    if let Err(e) = dto.is_valid() {
        return Err(e);
    }

    let api_status = state.api_state.api_status.read().await;
    if *api_status == ApiStatus::Maintenance.value() {
        return Err(ApiError {
            code: StatusCode::SERVICE_UNAVAILABLE,
            message: "Service is undergoing maintenance, try again later.".to_string(),
        });
    }

    // let get_input_media_if_any_result = get_input_media_if_any(dto, claims, state).await;
    // let Ok(input_media) = get_input_media_if_any_result
    // else {
    //     return Err(get_input_media_if_any_result.unwrap_err());
    // };

    let get_generate_media_request_result = get_generate_media_request(dto, claims, state).await;
    let Ok(generate_media_request) = get_generate_media_request_result
    else {
        return Err(get_generate_media_request_result.unwrap_err());
    };

    let req = generate_media_request.clone();
    let state = state.clone();

    let model = dto.model.clone().unwrap_or(dto.default_model().to_string());

    match dto.generator.as_ref() {
        MediaGenerator::MIST => match model.as_ref() {
            MediaModel::STABLE_DIFFUSION_1_5 => {
                replicate::service::spawn_generate_media_task(req, state)
                // modal::service::spawn_generate_media_task(req, state)
            }
            MediaModel::STABLE_DIFFUSION_2_1 => {
                replicate::service::spawn_generate_media_task(req, state)
            }
            MediaModel::OPENJOURNEY => replicate::service::spawn_generate_media_task(req, state),
            _ => return Err(DefaultApiError::InternalServerError.value()),
        },
        MediaGenerator::STABLE_HORDE => {
            stable_horde::service::spawn_generate_media_task(req, state)
        }
        MediaGenerator::DALLE => dalle::service::spawn_generate_media_task(req, state),
        _ => return Err(DefaultApiError::InternalServerError.value()),
    }

    Ok(generate_media_request)
}

async fn get_input_media_if_any(
    dto: &GenerateMediaDto,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<Option<Media>, ApiError> {
    let Some(id) = &dto.input_media_id
    else {
        return Ok(None);
    };

    match get_media_by_id(id, claims, &state.pool).await {
        Ok(media) => {
            if media.width as u16 != dto.width || media.height as u16 != dto.height {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: "Input image must have the same dimensions as request.".to_string(),
                });
            } else {
                Ok(Some(media))
            }
        }
        Err(e) => return Err(e),
    }
}

async fn get_generate_media_request(
    dto: &GenerateMediaDto,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<GenerateMediaRequest, ApiError> {
    let user = match users::service::get_user_by_id_as_admin(&claims.id, &state.pool).await {
        Ok(user) => user,
        Err(e) => return Err(e),
    };

    let ink_cost = users::util::ink::ink::calculate_ink_cost(&dto, None);

    if (user.ink - user.ink_pending) < ink_cost {
        return Err(ApiError {
            code: StatusCode::NOT_ACCEPTABLE,
            message: "Not enough ink.".to_string(),
        });
    }

    let openai_moderation_response_result =
        openai::moderation::check_prompt(&dto.prompt, &state.envy.openai_api_key).await;
    if let Err(e) = openai_moderation_response_result {
        tracing::error!("{:?}", e);
    } else {
        let openai_moderation_response = openai_moderation_response_result.unwrap();

        tracing::debug!("{:?}", openai_moderation_response);

        if let Some(result) = openai_moderation_response.results.first() {
            if result.flagged {
                tracing::info!("prompt was flagged: {:?}", dto.prompt);

                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: "Your prompt was flagged for inappropriate content.".to_string(),
                });
            }
        }
    }

    let Ok(mut tx) = state.pool.begin().await
    else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to begin transaction.".to_string(),
        });
    };

    let edit_user_ink_dto = EditUserInkDto {
        ink_increase: None,
        ink_decrease: None,
        ink_sum_increase: None,
        ink_sum_decrease: None,
        ink_pending_increase: Some(ink_cost),
        ink_pending_decrease: None,
    };

    let edit_user_ink_by_id_result =
        users::util::ink::ink::edit_user_ink_by_id(&claims.id, &edit_user_ink_dto, &mut tx).await;

    if edit_user_ink_by_id_result.is_err() {
        let rollback_result = tx.rollback().await;

        if let Some(e) = rollback_result.err() {
            tracing::error!(%e);
        } else {
            tracing::warn!("rolled back edit_user_ink_by_id_result");
        }

        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to complete transaction.".to_string(),
        });
    }

    let create_request_result =
        generate_media_requests::service::create_request(dto, claims, &mut tx).await;

    if create_request_result.is_err() {
        let rollback_result = tx.rollback().await;

        if let Some(e) = rollback_result.err() {
            tracing::error!(%e);
        } else {
            tracing::warn!("rolled back create_reqest_result");
        }

        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to complete transaction.".to_string(),
        });
    }

    match tx.commit().await {
        Ok(_) => return Ok(create_request_result.unwrap()),
        Err(e) => {
            tracing::error!(%e);
            return Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "An error occurred.".to_string(),
            });
        }
    }
}

pub async fn on_generate_media_completion_with_retry(
    generate_media_request: &GenerateMediaRequest,
    status: &GenerateMediaRequestStatus,
    media: &Option<Vec<Media>>,
    state: &Arc<AppState>,
) -> Result<(), ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(6);

    Retry::spawn(retry_strategy, || async {
        on_generate_media_completion(generate_media_request, status, media, state).await
    })
    .await
}

async fn on_generate_media_completion(
    generate_media_request: &GenerateMediaRequest,
    status: &GenerateMediaRequestStatus,
    media: &Option<Vec<Media>>,
    state: &Arc<AppState>,
) -> Result<(), ApiError> {
    let uid = &generate_media_request.user_id;

    let Ok(mut tx) = state.pool.begin().await
    else {
        tracing::warn!("on_generate_media_completion failed to begin pool transaction");
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to begin pool transaction.".to_string()
        });
    };

    let edit_generate_media_request_by_id_as_tx_result =
        generate_media_requests::service::edit_generate_media_request_by_id_as_tx_as_admin(
            &generate_media_request.id,
            status,
            &mut tx,
        )
        .await;

    if edit_generate_media_request_by_id_as_tx_result.is_err() {
        let rollback_result = tx.rollback().await;

        if let Some(e) = rollback_result.err() {
            tracing::error!("on_generate_media_completion failed to rollback edit_generate_media_request_by_id_as_tx_result: {:?}", e);
        } else {
            tracing::warn!("on_generate_media_completion rolled back edit_generate_media_request_by_id_as_tx_result");
        }

        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to complete edit_generate_media_request_by_id_as_tx".to_string(),
        });
    }

    let media_generated: u8 = match media {
        Some(media) => media.len() as u8,
        None => 0,
    };

    let ink_cost_original =
        users::util::ink::ink::calculate_ink_cost(&generate_media_request.generate_media_dto, None);

    let ink_cost_actual = users::util::ink::ink::calculate_ink_cost(
        &generate_media_request.generate_media_dto,
        Some(media_generated),
    );

    let edit_user_ink_dto = EditUserInkDto {
        ink_increase: None,
        ink_decrease: match ink_cost_actual > 0 {
            true => Some(ink_cost_actual),
            false => None,
        },
        ink_sum_increase: None,
        ink_sum_decrease: None,
        ink_pending_increase: None,
        ink_pending_decrease: Some(ink_cost_original),
    };

    let edit_user_ink_by_id_result =
        users::util::ink::ink::edit_user_ink_by_id(uid, &edit_user_ink_dto, &mut tx).await;

    if edit_user_ink_by_id_result.is_err() {
        let rollback_result = tx.rollback().await;

        if let Some(e) = rollback_result.err() {
            tracing::error!(
                "on_generate_media_completion failed to roll back edit_user_ink_by_id_result: {:?}",
                e
            );
        } else {
            tracing::warn!("on_generate_media_completion rolled back edit_user_ink_by_id_result");
        }

        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to complete edit_user_ink_by_id".to_string(),
        });
    }

    if let Err(e) = tx.commit().await {
        tracing::warn!("on_generate_media_completion failed to commit tx: {:?}", e);
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to commit tx".to_string(),
        });
    }

    if let Some(media) = media {
        if let Some(post) = Post::from_media(media.clone()) {
            devices::service::send_notifications_to_devices_with_user_id(
                "Mist".to_string(),
                "Your images are ready!".to_string(),
                Some(format!("post_view {}", post.id)),
                uid.to_string(),
                state.clone(),
            );

            posts::service::create_post_as_admin(post, &state.pool).await;
        }
    }

    Ok(())
}

pub async fn import_media(
    multipart: Multipart,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<Vec<Media>, ApiError> {
    let files_properties = get_files_properties(multipart).await;

    if files_properties.len() == 0 {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: "Received nothing to upload.".to_string(),
        });
    }

    for file_properties in &files_properties {
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

    let mut futures = Vec::with_capacity(files_properties.len());

    for file_properties in &files_properties {
        futures.push(upload_image_from_import_and_create_media(
            file_properties,
            claims,
            state,
        ));
    }

    let results = futures::future::join_all(futures).await;
    let mut media = Vec::with_capacity(files_properties.len());

    for result in results {
        if result.is_ok() {
            media.push(result.unwrap());
        }
    }

    if media.len() == 0 {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to upload files.".to_string(),
        });
    }

    match upload_media_with_retry(&media, &state.pool).await {
        Ok(m) => Ok(m),
        Err(e) => {
            tracing::error!("import_media failed upload_media_with_retry: {:?}", e);
            Err(e)
        }
    }
}

async fn upload_image_from_import_and_create_media(
    file_properties: &FileProperties,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<Media, ApiError> {
    let Ok(size) = imagesize::blob_size(&file_properties.data)
    else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to get image size.".to_string()
        });
    };

    let sub_folder = Some(["media/", &claims.id].concat());
    match backblaze::service::upload_file_with_retry(file_properties, &sub_folder, &state.b2).await
    {
        Ok(response) => {
            let b2_download_url = &state.b2.read().await.download_url;

            Ok(Media::from_import(
                &file_properties.id,
                &response,
                &size,
                claims,
                b2_download_url,
            ))
        }
        Err(e) => {
            tracing::error!(
                "upload_image_from_import_and_create_media failed upload_file_with_retry: {:?}",
                e
            );
            Err(e)
        }
    }
}

pub async fn upload_media_with_retry(
    media: &Vec<Media>,
    pool: &PgPool,
) -> Result<Vec<Media>, ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(3);

    Retry::spawn(retry_strategy, || async {
        upload_media(media.clone(), pool).await
    })
    .await
}

async fn upload_media(media: Vec<Media>, pool: &PgPool) -> Result<Vec<Media>, ApiError> {
    // IMPORTANT NOTE
    // due to the genius who made this (that's me...)
    // you need to update the num_properties to match the number of
    // properties inserted into the database

    // EXPLANATION
    // because we can upload multiple media at once
    // we need to insert num_properties * media.len()
    // therefore we loop to map each binding to a VALUE number

    let num_properties: u8 = 13;

    let mut sql = "
    INSERT INTO media (
        id, user_id, file_id, post_id, url,
        width, height, mime_type,
        generate_media_dto, seed, source, model, created_at
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
        sqlx = sqlx.bind(&m.post_id);
        sqlx = sqlx.bind(&m.url);
        sqlx = sqlx.bind(m.width.to_owned() as i16);
        sqlx = sqlx.bind(m.height.to_owned() as i16);
        sqlx = sqlx.bind(&m.mime_type);
        sqlx = sqlx.bind(&m.generate_media_dto);
        sqlx = sqlx.bind(&m.seed);
        sqlx = sqlx.bind(&m.source);
        sqlx = sqlx.bind(&m.model);
        sqlx = sqlx.bind(m.created_at.to_owned() as i64);
    }

    match sqlx.execute(pool).await {
        Ok(_) => Ok(media),
        Err(e) => {
            tracing::warn!("upload_media: {:?}", e);
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
    if let Some(source) = &dto.source {
        sqlx = sqlx.bind(source);
    }
    if let Some(model) = &dto.model {
        sqlx = sqlx.bind(model);
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

pub async fn get_media_by_id_as_anonymous(id: &str, pool: &PgPool) -> Result<Media, ApiError> {
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
            tracing::error!("get_media_by_id: {:?}", e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn delete_media_by_id(
    id: &str,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<(), ApiError> {
    let get_media_by_id_result = get_media_by_id_as_anonymous(id, &state.pool).await;
    let Ok(media) = get_media_by_id_result
    else {
        return Err(get_media_by_id_result.unwrap_err());
    };

    if media.user_id != claims.id {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: "Permission denied.".to_string(),
        });
    }

    let file_name = ["media/", &claims.id, "/", &media.id].concat();
    match backblaze::service::delete_file(&file_name, &media.file_id, &state.b2).await {
        Ok(_) => {
            let dto = media.generate_media_dto;

            if media.source == MediaGenerator::STABLE_HORDE && dto.is_some() {
                match delete_media_and_refund_ink(
                    &media.id,
                    &media.user_id,
                    &dto.unwrap().0,
                    &state.pool,
                )
                .await
                {
                    Ok(_) => {
                        if let Some(post_id) = media.post_id {
                            let media_id = media.id.to_string();
                            let state = state.clone();
                            posts::service::spawn_on_delete_post_media(post_id, media_id, state);
                        }

                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            } else {
                let sqlx_result = sqlx::query(
                    "
                    DELETE FROM media WHERE id = $1 AND user_id = $2
                    ",
                )
                .bind(id)
                .bind(&media.user_id)
                .execute(&state.pool)
                .await;

                match sqlx_result {
                    Ok(result) => match result.rows_affected() > 0 {
                        true => {
                            if let Some(post_id) = media.post_id {
                                let media_id = media.id.to_string();
                                let state = state.clone();
                                posts::service::spawn_on_delete_post_media(
                                    post_id, media_id, state,
                                );
                            }

                            Ok(())
                        }
                        false => Err(MediaApiError::MediaNotFound.value()),
                    },
                    Err(e) => {
                        tracing::error!(%e);
                        Err(DefaultApiError::InternalServerError.value())
                    }
                }
            }
        }
        Err(e) => Err(e),
    }
}

async fn delete_media_and_refund_ink(
    media_id: &str,
    user_id: &str,
    dto: &GenerateMediaDto,
    pool: &PgPool,
) -> Result<(), ApiError> {
    let Ok(mut tx) = pool.begin().await
    else {
        tracing::error!("delete_media_and_refund_ink failed to begin pool transaction");
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to begin transaction.".to_string()
        });
    };

    let delete_media_by_id_as_tx_result = delete_media_by_id_as_tx(media_id, &mut tx).await;

    if delete_media_by_id_as_tx_result.is_err() {
        let rollback_result = tx.rollback().await;

        if let Some(e) = rollback_result.err() {
            tracing::error!("delete_media_and_refund_ink_by_id failed to rollback delete_media_by_id_as_tx: {:?}", e);
        } else {
            tracing::warn!(
                "delete_media_and_refund_ink_by_id rolled back delete_media_by_id_as_tx"
            );
        }

        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to complete transaction.".to_string(),
        });
    }

    let ink_refunded = users::util::ink::ink::calculate_ink_cost(dto, Some(1));

    let edit_user_ink_dto = EditUserInkDto {
        ink_increase: match ink_refunded > 0 {
            true => Some(ink_refunded),
            false => None,
        },
        ink_decrease: None,
        ink_sum_increase: None,
        ink_sum_decrease: None,
        ink_pending_increase: None,
        ink_pending_decrease: None,
    };

    let edit_user_ink_by_id_result =
        users::util::ink::ink::edit_user_ink_by_id(user_id, &edit_user_ink_dto, &mut tx).await;

    if edit_user_ink_by_id_result.is_err() {
        let rollback_result = tx.rollback().await;

        if let Some(e) = rollback_result.err() {
            tracing::error!(
                "delete_media_and_refund_ink_by_id failed to roll back edit_user_ink_by_id_result: {:?}",
                e
            );
        } else {
            tracing::warn!(
                "delete_media_and_refund_ink_by_id rolled back edit_user_ink_by_id_result"
            );
        }

        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to complete transaction.".to_string(),
        });
    }

    if let Err(e) = tx.commit().await {
        tracing::error!(
            "delete_media_and_refund_ink_by_id failed to commit tx: {:?}",
            e
        );
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to complete transaction.".to_string(),
        });
    }

    Ok(())
}

async fn delete_media_by_id_as_tx(
    id: &str,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        DELETE FROM media WHERE id = $1
        ",
    )
    .bind(id)
    .execute(&mut *tx)
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
