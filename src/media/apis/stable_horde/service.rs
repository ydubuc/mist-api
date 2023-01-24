use std::{sync::Arc, time::Duration};

use bytes::Bytes;
use reqwest::{header, StatusCode};
use tokio::time::sleep;
use tokio_retry::{strategy::FixedInterval, Retry};
use uuid::Uuid;

use crate::{
    app::{
        errors::DefaultApiError, models::api_error::ApiError,
        util::multipart::models::file_properties::FileProperties,
    },
    auth::jwt::models::claims::Claims,
    generate_media_requests::{
        enums::generate_media_request_status::GenerateMediaRequestStatus,
        models::generate_media_request::GenerateMediaRequest,
    },
    media::{
        self, dtos::generate_media_dto::GenerateMediaDto, enums::media_model::MediaModel,
        models::media::Media, util::backblaze,
    },
    AppState,
};

use super::{
    config::API_URL,
    enums::stable_horde_model_version::StableHordeModelVersion,
    models::input_spec::{InputSpec, InputSpecParams},
    structs::{
        stable_horde_generate_async_response::StableHordeGenerateAsyncResponse,
        stable_horde_get_request_response::{StableHordeGeneration, StableHordeGetRequestResponse},
    },
};

pub fn spawn_generate_media_task(
    generate_media_request: GenerateMediaRequest,
    state: Arc<AppState>,
) {
    tokio::spawn(async move {
        let status: GenerateMediaRequestStatus;
        let media: Option<Vec<Media>>;

        match generate_media(&generate_media_request, &state).await {
            Ok(_media) => {
                status = GenerateMediaRequestStatus::Completed;
                media = Some(_media);
            }
            Err(_) => {
                status = GenerateMediaRequestStatus::Error;
                media = None;
            }
        }

        media::service::on_generate_media_completion_with_retry(
            &generate_media_request,
            &status,
            &media,
            &state,
        )
        .await
    });
}

async fn generate_media(
    request: &GenerateMediaRequest,
    state: &Arc<AppState>,
) -> Result<Vec<Media>, ApiError> {
    let stable_horde_api_key = &state.envy.stable_horde_api_key;
    let dto = &request.generate_media_dto;

    let stable_horde_request_response_result =
        await_request_completion(dto, stable_horde_api_key).await;
    let Ok(stable_horde_request_response) = stable_horde_request_response_result
    else {
        return Err(stable_horde_request_response_result.unwrap_err());
    };

    let Some(generations) = stable_horde_request_response.generations
    else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Stable Horde generated no images.".to_string()
        });
    };

    tracing::debug!(
        "request processed worker: {}, id: {}",
        generations.first().unwrap().worker_name,
        generations.first().unwrap().worker_id
    );

    let mut futures = Vec::with_capacity(generations.len());

    for generation in &generations {
        if generation.censored {
            continue;
        }

        futures.push(upload_image_and_create_media(request, generation, state));
    }

    let results = futures::future::join_all(futures).await;
    let mut media = Vec::with_capacity(generations.len());

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

    match media::service::upload_media_with_retry(&media, &state.pool).await {
        Ok(m) => Ok(m),
        Err(e) => {
            tracing::error!("generate_media failed upload_media_with_retry");
            Err(e)
        }
    }
}

async fn upload_image_and_create_media(
    request: &GenerateMediaRequest,
    stable_horde_generation: &StableHordeGeneration,
    state: &Arc<AppState>,
) -> Result<Media, ApiError> {
    let Ok(bytes) = get_bytes_with_retry(&stable_horde_generation.img).await
    else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to get bytes".to_string()
        });
    };

    let uuid = Uuid::new_v4().to_string();
    let file_properties = FileProperties {
        id: uuid.to_string(),
        field_name: uuid.to_string(),
        file_name: uuid.to_string(),
        mime_type: "image/webp".to_string(),
        data: Bytes::from(bytes),
    };

    let sub_folder = Some(["media/", &request.user_id].concat());
    match backblaze::service::upload_file_with_retry(&file_properties, &sub_folder, &state.b2).await
    {
        Ok(response) => {
            let b2_download_url = &state.b2.read().await.download_url;

            Ok(Media::from_request(
                &file_properties.id,
                request,
                Some(&stable_horde_generation.seed),
                &response,
                b2_download_url,
            ))
        }
        Err(e) => {
            tracing::error!("upload_image_and_create_media failed upload_file_with_retry");
            Err(e)
        }
    }
}

async fn get_bytes_with_retry(url: &str) -> Result<Bytes, ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(3);

    Retry::spawn(retry_strategy, || async { get_bytes(url).await }).await
}

async fn get_bytes(url: &str) -> Result<Bytes, ApiError> {
    let client = reqwest::Client::new();
    let result = client.get(url).send().await;

    match result {
        Ok(res) => match res.bytes().await {
            Ok(bytes) => Ok(bytes),
            Err(e) => {
                tracing::error!(%e);
                Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "Failed to get bytes from response.".to_string(),
                })
            }
        },
        Err(e) => {
            tracing::error!(%e);
            Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to get url response.".to_string(),
            })
        }
    }
}

async fn await_request_completion(
    dto: &GenerateMediaDto,
    stable_horde_api_key: &str,
) -> Result<StableHordeGetRequestResponse, ApiError> {
    let generate_async_result = generate_async_with_retry(dto, stable_horde_api_key).await;
    let Ok(generate_async_response) = generate_async_result
    else {
        tracing::error!("await_request_completion failed generate_async_with_retry");
        return Err(generate_async_result.unwrap_err());
    };

    let id = generate_async_response.id;

    sleep(Duration::from_millis(5000)).await;

    let Ok(initial_check_response) = get_request_by_id_with_retry(&id, true, stable_horde_api_key).await
    else {
        tracing::error!("await_request_completion failed get_request_by_id_with_retry (initial check)");
        return Err(DefaultApiError::InternalServerError.value());
    };

    if !initial_check_response.is_possible {
        tracing::error!(
            "await_request_completion failed (request is not possible): {:?}",
            dto
        );
        return Err(DefaultApiError::InternalServerError.value());
    }

    let mut request = initial_check_response;
    let mut encountered_error = false;

    let default_wait_time: u32 = 10;
    let max_wait_time: u32 = 60;

    let mut elapsed_time: u32 = 0;
    let mut wait_time: u32 = match request.wait_time > max_wait_time {
        true => max_wait_time,
        false => match request.wait_time > default_wait_time {
            true => request.wait_time,
            false => default_wait_time,
        },
    };

    while !request.done && !request.faulted && !encountered_error {
        tracing::debug!("waiting for request {}, estimated: {}", id, wait_time);
        sleep(Duration::from_secs(wait_time.into())).await;
        tracing::debug!("checking request {} after {}", id, wait_time);

        let Ok(check_response) = get_request_by_id_with_retry(&id, true, stable_horde_api_key).await
        else {
            tracing::error!("await_request_completion failed get_request_by_id_with_retry");
            encountered_error = true;
            continue;
        };

        request = check_response;
        elapsed_time += wait_time;
        wait_time = match request.wait_time > max_wait_time {
            true => max_wait_time,
            false => match request.wait_time > default_wait_time {
                true => request.wait_time,
                false => default_wait_time,
            },
        };

        if elapsed_time > 600 {
            tracing::error!("await_request_completion failed (ran out of time)");
            encountered_error = true;
            continue;
        }
    }

    if request.faulted {
        tracing::error!("await_request_completion failed (faulted): {:?}", request);
        return Err(DefaultApiError::InternalServerError.value());
    }
    if encountered_error {
        tracing::error!(
            "await_request_completion failed (encountered error): {:?}",
            request
        );
        return Err(DefaultApiError::InternalServerError.value());
    }

    let Ok(get_response) = get_request_by_id_with_retry(&id, false, stable_horde_api_key).await
    else {
        tracing::error!("await_request_completion failed get_request_by_id_with_retry (full)");
        return Err(DefaultApiError::InternalServerError.value());
    };

    Ok(get_response)
}

async fn generate_async_with_retry(
    dto: &GenerateMediaDto,
    stable_horde_api_key: &str,
) -> Result<StableHordeGenerateAsyncResponse, ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(3);

    Retry::spawn(retry_strategy, || async {
        generate_async(dto, stable_horde_api_key).await
    })
    .await
}

async fn generate_async(
    dto: &GenerateMediaDto,
    stable_horde_api_key: &str,
) -> Result<StableHordeGenerateAsyncResponse, ApiError> {
    let input_spec = provide_input_spec(dto);

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("apiKey", stable_horde_api_key.parse().unwrap());

    let client = reqwest::Client::new();
    let url = format!("{}/generate/async", API_URL);
    let result = client
        .post(url)
        .headers(headers)
        .json(&input_spec)
        .send()
        .await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(stable_horde_generate_async_response) => {
                    Ok(stable_horde_generate_async_response)
                }
                Err(_) => {
                    tracing::warn!("generate_async (1): {:?}", text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::warn!("generate_async (2): {:?}", e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::warn!("generate_async (3): {:?}", e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

async fn get_request_by_id_with_retry(
    id: &str,
    check_only: bool,
    stable_horde_api_key: &str,
) -> Result<StableHordeGetRequestResponse, ApiError> {
    let retry_strategy = FixedInterval::from_millis(match check_only {
        true => 10000,
        false => 31000,
    })
    .take(3);

    Retry::spawn(retry_strategy, || async {
        get_request_by_id(&id, check_only, stable_horde_api_key).await
    })
    .await
}

async fn get_request_by_id(
    id: &str,
    check_only: bool,
    stable_horde_api_key: &str,
) -> Result<StableHordeGetRequestResponse, ApiError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("apikey", stable_horde_api_key.parse().unwrap());

    let client = reqwest::Client::new();
    let check_param = if check_only { "check" } else { "status" };
    let url = format!("{}/generate/{}/{}", API_URL, check_param, id);
    let result = client.get(url).headers(headers).send().await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(stable_horde_get_request_response) => Ok(stable_horde_get_request_response),
                Err(_) => {
                    tracing::warn!("get_request_by_id (1): {:?}", text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::warn!("get_request_by_id (2): {:?}", e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::warn!("get_request_by_id (3): {:?}", e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

fn provide_input_spec(dto: &GenerateMediaDto) -> InputSpec {
    let model = dto.model.clone().unwrap_or(dto.default_model().to_string());
    let version = match model.as_ref() {
        MediaModel::STABLE_DIFFUSION_1_5 => StableHordeModelVersion::STABLE_DIFFUSION,
        MediaModel::STABLE_DIFFUSION_2_1 => StableHordeModelVersion::STABLE_DIFFUSION_2_1,
        _ => panic!("provide_input_spec for model {} not implemented.", model),
    };

    InputSpec {
        prompt: dto.prompt.to_string(),
        params: Some(InputSpecParams {
            sample_namer: None,
            cfg_scale: match dto.cfg_scale {
                Some(cfg_scale) => Some(cfg_scale as i8),
                None => None,
            },
            denoising_strength: None,
            seed: None,
            height: Some(dto.height),
            width: Some(dto.width),
            seed_variation: None,
            post_processing: Some(vec!["GFPGAN".to_string()]),
            karras: None,
            steps: Some(50),
            n: Some(dto.number),
        }),
        nsfw: Some(false),
        trusted_workers: Some(true),
        censor_nsfw: Some(true),
        workers: None,
        models: Some(vec![version.to_string()]),
        source_image: None,
        source_processing: None,
        source_mask: None,
        r2: Some(true),
    }
}

pub fn is_valid_model(model: &str) -> bool {
    let valid_models: [&str; 2] = [
        MediaModel::STABLE_DIFFUSION_1_5,
        MediaModel::STABLE_DIFFUSION_2_1,
    ];

    return valid_models.contains(&model);
}

pub fn is_valid_size(width: &u16, height: &u16) -> bool {
    let valid_widths: [u16; 3] = [512, 768, 1024];

    if !valid_widths.contains(width) {
        return false;
    }

    let valid_heights: [u16; 3] = [512, 768, 1024];

    if !valid_heights.contains(height) {
        return false;
    }

    return true;
}

pub fn is_valid_number(number: u8) -> bool {
    return (number > 0) && (number < 9);
}
