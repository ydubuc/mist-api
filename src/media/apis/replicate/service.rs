use std::{sync::Arc, time::Duration};

use bytes::Bytes;
use reqwest::{header, StatusCode};
use serde_json::Value;
use tokio::time::sleep;
use tokio_retry::{strategy::FixedInterval, Retry};
use uuid::Uuid;

use crate::{
    app::{
        errors::DefaultApiError, models::api_error::ApiError,
        util::multipart::models::file_properties::FileProperties,
    },
    generate_media_requests::{
        enums::generate_media_request_status::GenerateMediaRequestStatus,
        models::generate_media_request::GenerateMediaRequest,
    },
    media::{
        self, apis::replicate::enums::replicate_prediction_status::ReplicatePredictionStatus,
        dtos::generate_media_dto::GenerateMediaDto, enums::media_model::MediaModel,
        models::media::Media, util::backblaze,
    },
    AppState,
};

use super::{
    config::API_URL,
    enums::replicate_model_version::ReplicateModelVersion,
    models::{
        input_spec::InputSpec, input_spec_openjourney::InputSpecOpenjourney,
        input_spec_sd15::InputStableDiffusion15, input_spec_sd21::InputStableDiffusion21,
    },
    structs::replicate_predictions_response::ReplicatePredictionsResponse,
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
    let replicate_api_key = &state.envy.replicate_api_key;
    let dto = &request.generate_media_dto;

    let replicate_predictions_response_result =
        await_request_completion(dto, replicate_api_key).await;
    let Ok(replicate_predictions_response) = replicate_predictions_response_result
    else {
        return Err(replicate_predictions_response_result.unwrap_err());
    };

    let Some(urls) = replicate_predictions_response.output
    else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Replicate generated no images.".to_string()
        });
    };

    let seed = get_seed_from_logs(replicate_predictions_response.logs);

    let mut futures = Vec::with_capacity(urls.len());

    for url in &urls {
        futures.push(upload_image_and_create_media(
            request,
            url,
            match &seed {
                Some(seed) => Some(seed),
                None => None,
            },
            state,
        ));
    }

    let results = futures::future::join_all(futures).await;
    let mut media = Vec::with_capacity(urls.len());

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

fn get_seed_from_logs(logs: Option<String>) -> Option<String> {
    let Some(logs) = logs else { return None; };

    let mut splits = logs.split("\n");

    let Some(first) = splits.next() else { return None; };

    let prefix = "Using seed: ";
    if first.starts_with(prefix) {
        let Some(seed) = first.strip_prefix(prefix) else { return None;};
        return Some(seed.to_string());
    } else {
        return None;
    }
}

async fn upload_image_and_create_media(
    request: &GenerateMediaRequest,
    replicate_output_url: &str,
    seed: Option<&str>,
    state: &Arc<AppState>,
) -> Result<Media, ApiError> {
    let Ok(bytes) = get_bytes_with_retry(replicate_output_url, &state.envy.replicate_api_key).await
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
        mime_type: mime::IMAGE_PNG.to_string(),
        data: bytes,
    };

    let sub_folder = Some(["media/", &request.user_id].concat());
    match backblaze::service::upload_file_with_retry(&file_properties, &sub_folder, &state.b2).await
    {
        Ok(response) => {
            let b2_download_url = &state.b2.read().await.download_url;

            Ok(Media::from_request(
                &file_properties.id,
                request,
                seed,
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

async fn get_bytes_with_retry(url: &str, replicate_api_key: &str) -> Result<Bytes, ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(3);

    Retry::spawn(retry_strategy, || async {
        get_bytes(url, replicate_api_key).await
    })
    .await
}

async fn get_bytes(url: &str, replicate_api_key: &str) -> Result<Bytes, ApiError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Token {}", replicate_api_key).parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let result = client.get(url).headers(headers).send().await;

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
    replicate_api_key: &str,
) -> Result<ReplicatePredictionsResponse, ApiError> {
    let replicate_predictions_response_result =
        create_prediction_with_retry(dto, replicate_api_key).await;
    let Ok(replicate_predictions_response) = replicate_predictions_response_result
    else {
        tracing::error!("await_request_completion failed create_prediction_with_retry");
        return Err(replicate_predictions_response_result.unwrap_err());
    };

    let mut request = replicate_predictions_response;
    let mut succeeded = false;
    let mut failed = false;
    let mut canceled = false;
    let mut encountered_error = false;

    let mut elapsed_time: u32 = 0;
    let mut wait_time: u32 = 5;

    while !succeeded && !failed && !canceled && !encountered_error {
        tracing::debug!(
            "waiting for request {}, estimated: {}",
            request.id,
            wait_time
        );
        sleep(Duration::from_secs(wait_time.into())).await;
        tracing::debug!("checking request {} after {}", request.id, wait_time);

        let Ok(check_response) = get_prediction_by_id_with_retry(&request.id, replicate_api_key).await
        else {
            tracing::error!("await_request_completion failed get_request_by_id_with_retry");
            encountered_error = true;
            continue;
        };

        request = check_response;
        elapsed_time += wait_time;
        wait_time = 5;

        if elapsed_time > 600 {
            tracing::error!("await_request_completion failed (ran out of time)");
            encountered_error = true;
            continue;
        }

        succeeded = request.status == ReplicatePredictionStatus::Succeeded.value();
        failed = request.status == ReplicatePredictionStatus::Failed.value();
        canceled = request.status == ReplicatePredictionStatus::Canceled.value();

        if failed || canceled {
            tracing::error!("await_request_completion failed (failed or canceled)");
            encountered_error = true;
        }
    }

    if encountered_error {
        tracing::error!(
            "await_request_completion failed (encountered error): {:?}",
            request
        );
        return Err(DefaultApiError::InternalServerError.value());
    }

    Ok(request)
}

async fn create_prediction_with_retry(
    dto: &GenerateMediaDto,
    replicate_api_key: &str,
) -> Result<ReplicatePredictionsResponse, ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(3);

    Retry::spawn(retry_strategy, || async {
        create_prediction(dto, replicate_api_key).await
    })
    .await
}

async fn create_prediction(
    dto: &GenerateMediaDto,
    replicate_api_key: &str,
) -> Result<ReplicatePredictionsResponse, ApiError> {
    let input_spec = provide_input_spec(dto);

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Token {}", replicate_api_key).parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let url = format!("{}/predictions", API_URL);
    let result = client
        .post(url)
        .headers(headers)
        .json(&input_spec)
        .send()
        .await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(replicate_predictions_response) => Ok(replicate_predictions_response),
                Err(_) => {
                    tracing::warn!("create_prediction (1): {:?}", text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::warn!("create_prediction (2): {:?}", e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::warn!("create_prediction (3): {:?}", e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

async fn get_prediction_by_id_with_retry(
    id: &str,
    replicate_api_key: &str,
) -> Result<ReplicatePredictionsResponse, ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(3);

    Retry::spawn(retry_strategy, || async {
        get_prediction_by_id(id, replicate_api_key).await
    })
    .await
}

async fn get_prediction_by_id(
    id: &str,
    replicate_api_key: &str,
) -> Result<ReplicatePredictionsResponse, ApiError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Token {}", replicate_api_key).parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let url = format!("{}/predictions/{}", API_URL, id);
    let result = client.get(url).headers(headers).send().await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(replicate_predictions_response) => Ok(replicate_predictions_response),
                Err(_) => {
                    tracing::warn!("get_prediction_by_id (1): {:?}", text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::warn!("get_prediction_by_id (2): {:?}", e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::warn!("get_prediction_by_id (3): {:?}", e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

fn provide_input_spec(dto: &GenerateMediaDto) -> InputSpec {
    let model = dto.model.clone().unwrap_or(dto.default_model().to_string());
    let version: String;

    let input: Value = match model.as_ref() {
        MediaModel::STABLE_DIFFUSION_1_5 => {
            version = ReplicateModelVersion::STABLE_DIFFUSION_1_5.to_string();

            serde_json::to_value(InputStableDiffusion15 {
                prompt: dto.prompt.to_string(),
                negative_prompt: dto.negative_prompt.clone(),
                width: dto.width,
                height: dto.height,
                num_outputs: dto.number,
                num_inference_steps: 50,
                guidance_scale: dto.cfg_scale.unwrap_or(8),
                scheduler: Some("K_EULER".to_string()),
                seed: None,
            })
            .unwrap()
        }
        MediaModel::STABLE_DIFFUSION_2_1 => {
            version = ReplicateModelVersion::STABLE_DIFFUSION_2_1.to_string();

            serde_json::to_value(InputStableDiffusion21 {
                prompt: dto.prompt.to_string(),
                negative_prompt: dto.negative_prompt.clone(),
                width: dto.width,
                height: dto.height,
                num_outputs: dto.number,
                num_inference_steps: 50,
                guidance_scale: dto.cfg_scale.unwrap_or(8),
                scheduler: Some("K_EULER".to_string()),
                seed: None,
            })
            .unwrap()
        }
        MediaModel::OPENJOURNEY => {
            version = ReplicateModelVersion::OPENJOURNEY.to_string();

            serde_json::to_value(InputSpecOpenjourney {
                prompt: match dto.prompt.starts_with("mdjrny-v4 style") {
                    true => dto.prompt.to_string(),
                    false => format!("mdjrny-v4 style {}", dto.prompt),
                },
                width: dto.width,
                height: dto.height,
                num_outputs: dto.number,
                num_inference_steps: 50,
                guidance_scale: dto.cfg_scale.unwrap_or(8),
                seed: None,
            })
            .unwrap()
        }
        _ => panic!("provide_input_spec for model {} not implemented.", model),
    };

    InputSpec {
        version,
        input,
        webhook_completed: None,
    }
}
