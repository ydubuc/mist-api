use std::{sync::Arc, time::Duration};

use reqwest::{header, StatusCode};
use tokio::time::sleep;
use tokio_retry::{strategy::FixedInterval, Retry};
use uuid::Uuid;

use crate::{
    app::{
        self, errors::DefaultApiError, models::api_error::ApiError,
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
    models::input_spec::InputSpec,
    structs::{
        labml_generate_response::LabmlGenerateResponse,
        labml_get_request_response::{LabmlGetRequestResponse, LabmlImage},
    },
};

pub fn spawn_generate_media_task(
    generate_media_request: GenerateMediaRequest,
    claims: Claims,
    state: Arc<AppState>,
) {
    tokio::spawn(async move {
        let status: GenerateMediaRequestStatus;
        let media: Option<Vec<Media>>;

        match generate_media(&generate_media_request, &claims, &state).await {
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
            &claims,
            &state,
        )
        .await
    });
}

async fn generate_media(
    request: &GenerateMediaRequest,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<Vec<Media>, ApiError> {
    let labml_api_key = &state.envy.labml_api_key;
    let dto = &request.generate_media_dto;

    let response_result = await_request_completion(dto, labml_api_key).await;
    let Ok(response) = response_result
    else {
        return Err(response_result.unwrap_err());
    };

    if response.images.len() < 1 {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "LabML generated no images.".to_string(),
        });
    }

    let mut futures = Vec::with_capacity(response.images.len());

    for image in &response.images {
        futures.push(upload_image_and_create_media(request, image, claims, state));
    }

    let results = futures::future::join_all(futures).await;
    let mut media = Vec::with_capacity(response.images.len());

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
    labml_image: &LabmlImage,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<Media, ApiError> {
    let Ok(bytes) = app::util::reqwest::get_bytes(&labml_image.image).await
    else {
        return Err(ApiError { code: StatusCode::INTERNAL_SERVER_ERROR, message: "Failed to get bytes".to_string() })
    };

    let uuid = Uuid::new_v4().to_string();
    let file_properties = FileProperties {
        id: uuid.to_string(),
        field_name: uuid.to_string(),
        file_name: uuid.to_string(),
        mime_type: mime::IMAGE_JPEG.to_string(),
        data: bytes,
    };

    let sub_folder = Some(["media/", &claims.id].concat());
    match backblaze::service::upload_file_with_retry(&file_properties, &sub_folder, &state.b2).await
    {
        Ok(response) => {
            let b2_download_url = &state.b2.read().await.download_url;

            Ok(Media::from_request(
                &file_properties.id,
                request,
                None,
                &response,
                claims,
                b2_download_url,
            ))
        }
        Err(e) => {
            tracing::error!("upload_image_and_create_media failed upload_file_with_retry");
            Err(e)
        }
    }
}

async fn await_request_completion(
    dto: &GenerateMediaDto,
    labml_api_key: &str,
) -> Result<LabmlGetRequestResponse, ApiError> {
    let generate_response_result = generate_with_retry(dto, labml_api_key).await;
    let Ok(generate_response) = generate_response_result
    else {
        tracing::error!("await_request_completion failed generate_with_retry");
        return Err(generate_response_result.unwrap_err());
    };

    let id = generate_response.job_id;
    let eta: u32 = match generate_response.eta > 0.0 {
        true => generate_response.eta as u32,
        false => 3,
    };

    sleep(Duration::from_millis((1000 * eta).into())).await;

    let Ok(initial_check_response) = get_request_by_id_with_retry(&id).await
    else {
        tracing::error!("await_request_completion failed get_request_by_id_with_retry (initial check)");
        return Err(DefaultApiError::InternalServerError.value());
    };

    let mut request = initial_check_response;
    let mut encountered_error = false;

    let default_wait_time: u32 = 3;
    let max_wait_time: u32 = 60;

    let eta: u32 = match request.eta > 0.0 {
        true => request.eta as u32,
        false => default_wait_time,
    };

    let mut elapsed_time: u32 = 0;
    let mut wait_time: u32 = match eta > max_wait_time {
        true => max_wait_time,
        false => match eta > default_wait_time {
            true => eta,
            false => default_wait_time,
        },
    };

    while !request.is_completed && !encountered_error {
        tracing::debug!("waiting for request {}, estimated: {}", id, wait_time);
        sleep(Duration::from_secs(wait_time.into())).await;
        tracing::debug!("checking request {} after {}", id, wait_time);

        let Ok(check_response) = get_request_by_id_with_retry(&id).await
        else {
            tracing::error!("await_request_completion failed get_request_by_id_with_retry");
            encountered_error = true;
            continue;
        };

        request = check_response;
        elapsed_time += wait_time;

        let eta: u32 = match request.eta > 0.0 {
            true => request.eta as u32,
            false => default_wait_time,
        };

        wait_time = match eta > max_wait_time {
            true => max_wait_time,
            false => match eta > default_wait_time {
                true => eta,
                false => default_wait_time,
            },
        };

        if elapsed_time > 600 {
            tracing::error!("await_request_completion failed (ran out of time)");
            encountered_error = true;
            continue;
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

async fn generate_with_retry(
    dto: &GenerateMediaDto,
    labml_api_key: &str,
) -> Result<LabmlGenerateResponse, ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(3);

    Retry::spawn(retry_strategy, || async {
        match generate(dto, labml_api_key).await {
            Ok(response) => match response.is_success {
                true => Ok(response),
                false => Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "generate_with_retry failed (not is_success)".to_string(),
                }),
            },
            Err(e) => Err(e),
        }
    })
    .await
}

async fn generate(
    dto: &GenerateMediaDto,
    labml_api_key: &str,
) -> Result<LabmlGenerateResponse, ApiError> {
    let input_spec = provide_input_spec(dto, labml_api_key);

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let client = reqwest::Client::new();
    let url = format!("{}/generate", API_URL);
    let result = client
        .post(url)
        .headers(headers)
        .json(&input_spec)
        .send()
        .await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(labml_generate_response) => Ok(labml_generate_response),
                Err(_) => {
                    tracing::warn!("generate (1): {:?}", text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::warn!("generate (2): {:?}", e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::warn!("generate (3): {:?}", e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

async fn get_request_by_id_with_retry(id: &str) -> Result<LabmlGetRequestResponse, ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(3);

    Retry::spawn(retry_strategy, || async {
        match get_request_by_id(&id).await {
            Ok(response) => match response.is_success {
                true => Ok(response),
                false => Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "get_request_by_id_with_retry failed (not is_success)".to_string(),
                }),
            },
            Err(e) => Err(e),
        }
    })
    .await
}

async fn get_request_by_id(id: &str) -> Result<LabmlGetRequestResponse, ApiError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let client = reqwest::Client::new();
    let url = format!("{}/status/{}", API_URL, id);
    let result = client.get(url).headers(headers).send().await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(labml_get_request_response) => Ok(labml_get_request_response),
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

fn provide_input_spec(dto: &GenerateMediaDto, labml_api_key: &str) -> InputSpec {
    InputSpec {
        api_token: labml_api_key.to_string(),
        prompt: dto.prompt.to_string(),
        negative_prompt: None,
        n_steps: Some(50),
        sampling_method: None,
        prompt_strength: None,
        seeds: None,
        source: None,
        mask: None,
        image_strength: None,
    }
}

pub fn is_valid_size(width: &u16, height: &u16) -> bool {
    let valid_widths: [u16; 1] = [512];

    if !valid_widths.contains(width) {
        return false;
    }

    let valid_heights: [u16; 1] = [512];

    if !valid_heights.contains(height) {
        return false;
    }

    return true;
}

pub fn is_valid_number(number: u8) -> bool {
    return number == 2;
}
