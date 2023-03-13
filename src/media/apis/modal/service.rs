use std::sync::Arc;

use bytes::Bytes;
use reqwest::{header, StatusCode};
use serde_json::Value;
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
    media::{self, enums::media_model::MediaModel, models::media::Media, util::backblaze},
    webhooks::modal::dtos::receive_webhook_dto::ReceiveWebhookDto,
    AppState,
};

use super::{
    config::api_url, models::input_spec_sd15::InputStableDiffusion15,
    structs::modal_entrypoint_response::ModalEntrypointResponse,
};

pub fn spawn_generate_media_task(
    generate_media_request: GenerateMediaRequest,
    state: Arc<AppState>,
) {
    tokio::spawn(async move {
        match call_modal_entrypoint_with_retry(&generate_media_request, &state).await {
            Err(e) => {
                tracing::error!(
                    "spawn_generate_media_task failed call_modal_entrypoint_with_retry: {:?}",
                    e
                );
            }
            _ => {}
        }
    });
}

pub fn on_receive_webhook(
    generate_media_request: GenerateMediaRequest,
    webhook_dto: ReceiveWebhookDto,
    state: Arc<AppState>,
) {
    tokio::spawn(async move {
        let status: GenerateMediaRequestStatus;
        let media: Option<Vec<Media>>;

        match generate_media(&generate_media_request, &webhook_dto, &state).await {
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
    webhook_dto: &ReceiveWebhookDto,
    state: &Arc<AppState>,
) -> Result<Vec<Media>, ApiError> {
    if webhook_dto.images.len() < 1 {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Modal generated no images.".to_string(),
        });
    };

    let mut futures = Vec::with_capacity(webhook_dto.images.len());

    for image in &webhook_dto.images {
        futures.push(upload_image_and_create_media(request, image, state));
    }

    let results = futures::future::join_all(futures).await;
    let mut media = Vec::with_capacity(webhook_dto.images.len());

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
    base64_image: &str,
    state: &Arc<AppState>,
) -> Result<Media, ApiError> {
    let Ok(bytes) = base64::decode(&base64_image)
    else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Could not decode image.".to_string()
        });
    };

    let uuid = Uuid::new_v4().to_string();
    let file_properties = FileProperties {
        id: uuid.to_string(),
        field_name: uuid.to_string(),
        file_name: uuid.to_string(),
        mime_type: mime::IMAGE_PNG.to_string(),
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
                None,
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

async fn call_modal_entrypoint_with_retry(
    request: &GenerateMediaRequest,
    state: &Arc<AppState>,
) -> Result<ModalEntrypointResponse, ApiError> {
    let retry_strategy = FixedInterval::from_millis(10000).take(3);

    Retry::spawn(retry_strategy, || async {
        call_modal_entrypoint(request, state).await
    })
    .await
}

async fn call_modal_entrypoint(
    request: &GenerateMediaRequest,
    state: &Arc<AppState>,
) -> Result<ModalEntrypointResponse, ApiError> {
    let modal_webhook_secret = &state.envy.modal_webhook_secret;
    let input_spec = provide_input_spec(request, state);
    let dto = &request.generate_media_dto;
    let model = dto.model.clone().unwrap_or(dto.default_model().to_string());

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Bearer {}", modal_webhook_secret).parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let result = client
        .post(api_url(&model))
        .headers(headers)
        .json(&input_spec)
        .send()
        .await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(modal_entrypoint_response) => Ok(modal_entrypoint_response),
                Err(_) => {
                    tracing::warn!("call_modal_entrypoint (1): {:?}", text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::warn!("call_modal_entrypoint (2): {:?}", e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::warn!("call_modal_entrypoint (3): {:?}", e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

fn provide_input_spec(request: &GenerateMediaRequest, state: &Arc<AppState>) -> Value {
    let dto = &request.generate_media_dto;

    tracing::debug!("{}", state.envy.railway_static_url);

    let input: Value = serde_json::to_value(InputStableDiffusion15 {
        request_id: request.id.to_string(),
        prompt: dto.prompt.to_string(),
        negative_prompt: dto.negative_prompt.clone(),
        width: dto.width,
        height: dto.height,
        number: dto.number,
        steps: 50,
        cfg_scale: dto.cfg_scale.unwrap_or(8),
        callback_url: format!("{}/webhooks/modal", state.envy.railway_static_url),
    })
    .unwrap();

    input
}
