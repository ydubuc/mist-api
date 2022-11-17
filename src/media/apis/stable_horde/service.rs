use std::time::Duration;

use bytes::Bytes;
use reqwest::{header, StatusCode};
use tokio::time::sleep;
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
        self, dtos::generate_media_dto::GenerateMediaDto, models::media::Media, util::backblaze,
    },
    AppState,
};

use super::{
    config::API_URL,
    models::input_spec::{InputSpec, InputSpecParams},
    structs::{
        stable_horde_generate_async_response::StableHordeGenerateAsyncResponse,
        stable_horde_get_request_response::StableHordeGetRequestResponse,
    },
};

pub fn spawn_generate_media_task(
    generate_media_request: GenerateMediaRequest,
    claims: Claims,
    state: AppState,
) {
    tokio::spawn(async move {
        let status: GenerateMediaRequestStatus;
        let media: Option<Vec<Media>>;

        match generate_media(&generate_media_request.generate_media_dto, &claims, &state).await {
            Ok(_media) => {
                status = GenerateMediaRequestStatus::Completed;
                media = Some(_media);
            }
            Err(_) => {
                status = GenerateMediaRequestStatus::Error;
                media = None;
            }
        }

        media::service::on_generate_media_completion(
            &generate_media_request,
            &status,
            &media,
            &claims,
            &state,
        )
        .await
    });
}

async fn generate_media(
    dto: &GenerateMediaDto,
    claims: &Claims,
    state: &AppState,
) -> Result<Vec<Media>, ApiError> {
    let stable_horde_api_key = &state.envy.stable_horde_api_key;

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
            message: "Stable horde generated no images.".to_string()
        });
    };

    println!(
        "request processed by {}",
        generations.first().unwrap().worker_name
    );

    let mut files_properties = Vec::with_capacity(generations.len());

    for generation in &generations {
        let Ok(bytes) = base64::decode(&generation.img)
        else {
            continue;
        };

        let uuid = Uuid::new_v4().to_string();
        let file_properties = FileProperties {
            id: uuid.to_string(),
            field_name: uuid.to_string(),
            file_name: uuid.to_string(),
            mime_type: "image/webp".to_string(),
            data: Bytes::from(bytes),
        };

        files_properties.push(file_properties);
    }

    let sub_folder = Some(["media/", &claims.id].concat());
    match backblaze::service::upload_files(&files_properties, &sub_folder, &state.b2).await {
        Ok(responses) => {
            let media = Media::from_dto(dto, &responses, claims, &state.b2);

            if media.len() == 0 {
                return Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "Failed to upload files.".to_string(),
                });
            }

            match media::service::upload_media(media, &state.pool).await {
                Ok(m) => Ok(m),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

async fn await_request_completion(
    dto: &GenerateMediaDto,
    stable_horde_api_key: &str,
) -> Result<StableHordeGetRequestResponse, ApiError> {
    let generate_async_result = generate_async(dto, stable_horde_api_key).await;
    let Ok(generate_async_response) = generate_async_result
    else {
        return Err(generate_async_result.unwrap_err());
    };

    let id = generate_async_response.id;

    sleep(Duration::from_millis(5000)).await;

    let Ok(initial_check_response) = get_request_by_id(&id, true, stable_horde_api_key).await
    else {
        tracing::error!("Failed to get request by id while awaiting stable horde request.");
        return Err(DefaultApiError::InternalServerError.value());
    };

    if !initial_check_response.is_possible {
        tracing::error!("Failed to generate stable horde request. Request is not possible.");
        return Err(DefaultApiError::InternalServerError.value());
    }

    let mut request = initial_check_response;
    let mut encountered_error = false;

    let default_wait_time: u32 = 10;
    let max_wait_time: u32 = 120;

    let mut wait_time: u32 = match request.wait_time > max_wait_time {
        true => max_wait_time,
        false => match request.wait_time > default_wait_time {
            true => request.wait_time,
            false => default_wait_time,
        },
    };

    while !request.done && !request.faulted && !encountered_error {
        println!("waiting for request, estimated wait_time: {}", wait_time);
        sleep(Duration::from_secs(wait_time.into())).await;

        println!("checking request after waiting for {}", wait_time);

        let Ok(check_response) = get_request_by_id(&id, true, stable_horde_api_key).await
        else {
            tracing::error!("Failed to get request by id while awaiting stable horde request.");
            encountered_error = true;
            continue;
        };

        request = check_response;

        wait_time = match request.wait_time > max_wait_time {
            true => max_wait_time,
            false => match request.wait_time > default_wait_time {
                true => request.wait_time,
                false => default_wait_time,
            },
        };
    }

    if request.faulted {
        tracing::error!("Stable horde task finished with error: {:?}", request);
        return Err(DefaultApiError::InternalServerError.value());
    }

    let Ok(get_response) = get_request_by_id(&id, false, stable_horde_api_key).await
    else {
        tracing::error!("Failed to get request full status by id for stable horde request.");
        return Err(DefaultApiError::InternalServerError.value());
    };

    Ok(get_response)
}

async fn generate_async(
    dto: &GenerateMediaDto,
    stable_horde_api_key: &str,
) -> Result<StableHordeGenerateAsyncResponse, ApiError> {
    let input_spec = match provide_input_spec(dto) {
        Ok(input_spec) => input_spec,
        Err(e) => return Err(e),
    };

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
                    tracing::error!(%text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::error!(%e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
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
                    tracing::error!(%text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::error!(%e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

fn provide_input_spec(dto: &GenerateMediaDto) -> Result<InputSpec, ApiError> {
    let size = (dto.width, dto.height);

    let valid_sizes = [
        (512, 512),
        (512, 1024),
        (1024, 512),
        (640, 1024),
        (1024, 640),
    ];

    if !valid_sizes.contains(&size) {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: [
                "Size must be one of: ",
                &valid_sizes
                    .map(|val| format!("{}x{}", val.0, val.1))
                    .join(", "),
            ]
            .concat(),
        });
    }

    Ok(InputSpec {
        prompt: dto.prompt.to_string(),
        params: Some(InputSpecParams {
            sample_namer: None,
            cfg_scale: None,
            denoising_strength: None,
            seed: None,
            height: Some(dto.height),
            width: Some(dto.width),
            seed_variation: None,
            use_gfpgan: Some(true),
            karras: None,
            use_real_esrgan: None,
            use_ldsr: None,
            use_upscaling: Some(false),
            steps: Some(50),
            n: Some(dto.number),
        }),
        nsfw: Some(false),
        trusted_workers: Some(false),
        censor_nsfw: Some(false),
        workers: None,
        // workers: Some(vec!["63cc5925-beb8-4e67-91d5-8cfe305d530a".to_string()]),
        models: None,
        source_image: None,
        source_processing: None,
        source_mask: None,
    })
}
