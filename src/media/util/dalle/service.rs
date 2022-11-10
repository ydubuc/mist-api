use serde_json::json;
use tracing::Level;
use uuid::Uuid;

extern crate reqwest;

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
        self, dtos::generate_media_dto::GenerateMediaDto, enums::media_source::MediaSource,
        models::media::Media, util::backblaze,
    },
    AppState,
};
use reqwest::{header, StatusCode};

use super::models::dalle_generate_image_response::DalleGenerateImageResponse;

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
    match dalle_generate_image(dto, &state.envy.openai_api_key).await {
        Ok(dalle_response) => {
            let mut files_properties = Vec::new();

            for data in &dalle_response.data {
                match app::util::reqwest::get_bytes(&data.url).await {
                    Ok(bytes) => {
                        let uuid = Uuid::new_v4().to_string();
                        let file_properties = FileProperties {
                            id: uuid.to_string(),
                            field_name: uuid.to_string(),
                            file_name: uuid.to_string(),
                            mime_type: mime::IMAGE_PNG,
                            data: bytes,
                        };

                        files_properties.push(file_properties);
                    }
                    Err(_) => {
                        // failed to get bytes
                        // skip to next data
                    }
                }
            }

            let sub_folder = Some(["media/", &claims.id].concat());
            match backblaze::service::upload_files(files_properties, &sub_folder, &state.b2).await {
                Ok(responses) => {
                    let media = media::service::create_media_from_responses(
                        responses,
                        MediaSource::Dalle,
                        claims,
                        &state.b2,
                    );

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
        Err(e) => Err(e),
    }
}

async fn dalle_generate_image(
    dto: &GenerateMediaDto,
    openai_api_key: &str,
) -> Result<DalleGenerateImageResponse, ApiError> {
    let size = [
        dto.width.to_string(),
        "x".to_string(),
        dto.height.to_string(),
    ]
    .concat();

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

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Bearer ", openai_api_key].concat().parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let result = client
        .post("https://api.openai.com/v1/images/generations")
        .headers(headers)
        .body(
            json!({
                "prompt": dto.prompt,
                "n": dto.number,
                "size": size,
                "response_format": "url"
            })
            .to_string(),
        )
        .send()
        .await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(dalle_response) => Ok(dalle_response),
                Err(_) => {
                    tracing::event!(Level::ERROR, %text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::event!(Level::ERROR, %e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::event!(Level::ERROR, %e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}
