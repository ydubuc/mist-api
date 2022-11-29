use std::sync::Arc;

use bytes::Bytes;
use reqwest::{header, Response, StatusCode};
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
    models::input_spec::InputSpec,
    structs::mist_stability_generate_images_response::{
        MistStabilityGenerateImagesResponse, MistStabilityImageData,
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

        match generate_media(&generate_media_request.generate_media_dto, &claims, &state).await {
            Ok(m) => {
                status = GenerateMediaRequestStatus::Completed;
                media = Some(m);
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
    state: &Arc<AppState>,
) -> Result<Vec<Media>, ApiError> {
    let mist_stability_generate_images_result =
        mist_stability_generate_images(dto, &state.envy.mist_stability_api_key).await;
    let Ok(mist_response) = mist_stability_generate_images_result
    else {
        return Err(mist_stability_generate_images_result.unwrap_err());
    };

    let mut futures = Vec::with_capacity(mist_response.data.len());

    for data in &mist_response.data {
        futures.push(upload_image_and_create_media(dto, data, claims, state));
    }

    let results = futures::future::join_all(futures).await;
    let mut media = Vec::with_capacity(mist_response.data.len());

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
        Err(e) => Err(e),
    }
}

async fn upload_image_and_create_media(
    dto: &GenerateMediaDto,
    mist_stability_image_data: &MistStabilityImageData,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<Media, ApiError> {
    let Ok(bytes) = base64::decode(&mist_stability_image_data.base64)
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
        mime_type: "image/webp".to_string(),
        data: Bytes::from(bytes),
    };

    let sub_folder = Some(["media/", &claims.id].concat());
    match backblaze::service::upload_file_with_retry(&file_properties, &sub_folder, &state.b2).await
    {
        Ok(response) => {
            let b2_download_url = &state.b2.read().await.download_url;

            Ok(Media::from_dto(
                dto,
                Some(&mist_stability_image_data.seed),
                &response,
                claims,
                b2_download_url,
            ))
        }
        Err(e) => Err(e),
    }
}

async fn mist_stability_generate_images(
    dto: &GenerateMediaDto,
    mist_stability_api_key: &str,
) -> Result<MistStabilityGenerateImagesResponse, ApiError> {
    let input_spec = provide_input_spec(dto);

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Bearer ", mist_stability_api_key]
            .concat()
            .parse()
            .unwrap(),
    );

    let client = reqwest::Client::new();
    let url = format!("{}/images/generate", API_URL);
    let result = client
        .post(url)
        .headers(headers)
        .json(&input_spec)
        .send()
        .await;

    match result {
        Ok(res) => parse_response_to_mist_stability_generate_images_response(res).await,
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

fn provide_input_spec(dto: &GenerateMediaDto) -> InputSpec {
    InputSpec {
        prompt: dto.prompt.to_string(),
        width: dto.width,
        height: dto.height,
        number: dto.number,
        steps: Some(50),
    }
}

async fn parse_response_to_mist_stability_generate_images_response(
    res: Response,
) -> Result<MistStabilityGenerateImagesResponse, ApiError> {
    match res.text().await {
        Ok(text) => match serde_json::from_str(&text) {
            Ok(mist_stability_generate_images_response) => {
                Ok(mist_stability_generate_images_response)
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
    }
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
