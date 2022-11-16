use axum::http::StatusCode;
use bytes::Bytes;
use reqwest::{header, Response};
use uuid::Uuid;

extern crate reqwest;

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
        self, dtos::generate_media_dto::GenerateMediaDto, enums::media_source::MediaSource,
        models::media::Media, util::backblaze,
    },
    AppState,
};

use super::{
    config::API_URL, models::input_spec::InputSpec,
    structs::dalle_generate_image_response::DalleGenerateImageResponse,
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
            Ok(m) => {
                status = GenerateMediaRequestStatus::Completed;
                media = Some(m);
            }
            Err(e) => {
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
    let dalle_generate_images_result = dalle_generate_images(dto, &state.envy.openai_api_key).await;
    let Ok(dalle_response) = dalle_generate_images_result
    else {
        return Err(dalle_generate_images_result.unwrap_err());
    };

    let mut files_properties = Vec::new();

    for data in &dalle_response.data {
        let Ok(bytes) = base64::decode(&data.b64_json)
        else {
            continue;
        };

        let uuid = Uuid::new_v4().to_string();
        let file_properties = FileProperties {
            id: uuid.to_string(),
            field_name: uuid.to_string(),
            file_name: uuid.to_string(),
            mime_type: mime::IMAGE_PNG.to_string(),
            data: Bytes::from(bytes),
        };

        files_properties.push(file_properties);
    }

    let sub_folder = Some(["media/", &claims.id].concat());
    match backblaze::service::upload_files(&files_properties, &sub_folder, &state.b2).await {
        Ok(responses) => {
            let media =
                Media::from_backblaze_responses(responses, MediaSource::Dalle, claims, &state.b2);

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

async fn dalle_generate_images(
    dto: &GenerateMediaDto,
    openai_api_key: &str,
) -> Result<DalleGenerateImageResponse, ApiError> {
    let input_spec = match provide_input_spec(dto) {
        Ok(input_spec) => input_spec,
        Err(e) => return Err(e),
    };

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Bearer ", openai_api_key].concat().parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let url = format!("{}/images/generations", API_URL);
    let result = client
        .post(url)
        .headers(headers)
        .json(&input_spec)
        .send()
        .await;

    match result {
        Ok(res) => parse_response_to_dalle_generate_image_response(res).await,
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

fn provide_input_spec(dto: &GenerateMediaDto) -> Result<InputSpec, ApiError> {
    let size = format!("{}x{}", dto.width, dto.height);

    let valid_sizes = [
        "256x256".to_string(),
        "512x512".to_string(),
        "1024x1024".to_string(),
    ];

    if !valid_sizes.contains(&size) {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: ["Size must be one of: ", &valid_sizes.join(",")].concat(),
        });
    }

    Ok(InputSpec {
        prompt: dto.prompt.to_string(),
        n: dto.number,
        size,
        response_format: "b64_json".to_string(),
        // response_format: "url".to_string(),
    })
}

async fn parse_response_to_dalle_generate_image_response(
    res: Response,
) -> Result<DalleGenerateImageResponse, ApiError> {
    match res.text().await {
        Ok(text) => match serde_json::from_str(&text) {
            Ok(dalle_generate_image_response) => Ok(dalle_generate_image_response),
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
