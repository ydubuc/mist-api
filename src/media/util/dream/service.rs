use std::time::Duration;

use axum::http::StatusCode;
use reqwest::{header, Response};
use serde_json::json;
use tokio::time::sleep;
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
        self,
        dtos::generate_media_dto::GenerateMediaDto,
        models::media::Media,
        util::{backblaze, dream::enums::dream_task_state::DreamTaskState},
    },
    AppState,
};

use super::{
    config::API_URL, models::input_spec::InputSpec, structs::dream_task_response::DreamTaskResponse,
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
    let dream_api_key = &state.envy.dream_api_key;

    let dream_responses = await_tasks(dto, state).await;

    let mut files_properties = Vec::new();
    let mut failures = Vec::new();

    for response in &dream_responses {
        let Ok(res) = response
        else {
            failures.push(response);
            continue;
        };
        let Some(url) = &res.result
        else {
            failures.push(response);
            continue;
        };

        match app::util::reqwest::get_bytes(&url).await {
            Ok(bytes) => {
                let uuid = Uuid::new_v4().to_string();
                let file_properties = FileProperties {
                    id: uuid.to_string(),
                    field_name: uuid.to_string(),
                    file_name: uuid.to_string(),
                    mime_type: mime::IMAGE_JPEG.to_string(),
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
    match backblaze::service::upload_files(&files_properties, &sub_folder, &state.b2).await {
        Ok(responses) => {
            // let media =
            //     Media::from_backblaze_responses(responses, MediaSource::Dream, claims, &state.b2);
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

pub async fn await_tasks(
    dto: &GenerateMediaDto,
    state: &AppState,
) -> Vec<Result<DreamTaskResponse, ApiError>> {
    let mut futures = Vec::new();

    for _ in 0..dto.number {
        futures.push(await_task_completion(&dto, &state.envy.dream_api_key));
    }

    futures::future::join_all(futures).await
}

async fn await_task_completion(
    dto: &GenerateMediaDto,
    dream_api_key: &str,
) -> Result<DreamTaskResponse, ApiError> {
    let create_task_result = create_task(dream_api_key).await;
    let Ok(create_task_response) = create_task_result
    else {
        return Err(create_task_result.unwrap_err());
    };

    let update_task_result = update_task_by_id(&create_task_response.id, dto, dream_api_key).await;
    let Ok(update_task_response) = update_task_result
    else {
        return Err(update_task_result.unwrap_err())
    };

    let mut task = update_task_response;
    let mut encountered_error = false;

    while (task.state == DreamTaskState::GENERATING || task.state == DreamTaskState::PENDING)
        && !encountered_error
    {
        sleep(Duration::from_millis(5000)).await;

        let Ok(task_response) = get_task_by_id(&task.id, dream_api_key).await
        else {
            tracing::error!("Failed to get task by id while awaiting dream task.");
            encountered_error = true;
            continue;
        };

        task = task_response;
    }

    if task.state != DreamTaskState::COMPLETED || encountered_error {
        tracing::error!("Dream task finished with error: {:?}", task);
        return Err(DefaultApiError::InternalServerError.value());
    }

    Ok(task)
}

async fn create_task(dream_api_key: &str) -> Result<DreamTaskResponse, ApiError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Bearer ", dream_api_key].concat().parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let url = format!("{}/tasks", API_URL);
    let result = client
        .post(url)
        .headers(headers)
        .json(&json!({
            "use_target_image": false,
        }))
        .send()
        .await;

    match result {
        Ok(res) => parse_response_to_dream_task_response(res).await,
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

async fn get_task_by_id(id: &str, dream_api_key: &str) -> Result<DreamTaskResponse, ApiError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Bearer ", dream_api_key].concat().parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let url = format!("{}/tasks/{}", API_URL, id);
    let result = client.get(url).headers(headers).send().await;

    match result {
        Ok(res) => parse_response_to_dream_task_response(res).await,
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

async fn update_task_by_id(
    id: &str,
    dto: &GenerateMediaDto,
    dream_api_key: &str,
) -> Result<DreamTaskResponse, ApiError> {
    let input_spec = match provide_input_spec(dto) {
        Ok(input_spec) => input_spec,
        Err(e) => return Err(e),
    };

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Bearer ", dream_api_key].concat().parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let result = client
        .put(format!("https://api.luan.tools/api/tasks/{}", id))
        .headers(headers)
        .json(&json!({ "input_spec": &input_spec }))
        .send()
        .await;

    match result {
        Ok(res) => parse_response_to_dream_task_response(res).await,
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
        style: 3, // TODO: add style to dto
        prompt: dto.prompt.to_string(),
        target_image_weight: None, // TODO: add image weight to dto
        width: Some(dto.width),
        height: Some(dto.height),
    })
}

async fn parse_response_to_dream_task_response(
    res: Response,
) -> Result<DreamTaskResponse, ApiError> {
    match res.text().await {
        Ok(text) => match serde_json::from_str(&text) {
            Ok(dream_task_response) => Ok(dream_task_response),
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
