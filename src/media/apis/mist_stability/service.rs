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
    config::API_URL, models::input_spec::InputSpec,
    structs::mist_stability_generate_images_response::MistStabilityGenerateImagesResponse,
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
    let mist_stability_generate_images_result =
        mist_stability_generate_images(dto, &state.envy.mist_stability_api_key).await;
    let Ok(mist_response) = mist_stability_generate_images_result
    else {
        return Err(mist_stability_generate_images_result.unwrap_err());
    };

    let mut files_properties = Vec::new();

    for data in &mist_response.base64_data {
        let Ok(bytes) = base64::decode(&data)
        else {
            println!("could not decode data");
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

    println!("files properties {}", files_properties.len());

    let sub_folder = Some(["media/", &claims.id].concat());
    match backblaze::service::upload_files(&files_properties, &sub_folder, &state.b2).await {
        Ok(responses) => {
            println!("responses {}", responses.len());

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
    let valid_widths: [u16; 3] = [512, 640, 1024];

    if !valid_widths.contains(width) {
        return false;
    }

    let valid_heights: [u16; 3] = [512, 640, 1024];

    if !valid_heights.contains(height) {
        return false;
    }

    return true;
}
