use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use axum_macros::debug_handler;
use validator::Validate;

use crate::{
    app::{models::api_error::ApiError, structs::json_from_request::JsonFromRequest},
    auth::jwt::models::claims::Claims,
    generate_media_requests::models::generate_media_request::GenerateMediaRequest,
    AppState,
};

use super::{
    dtos::{generate_media_dto::GenerateMediaDto, get_media_filter_dto::GetMediaFilterDto},
    models::media::Media,
    service,
};

#[debug_handler]
pub async fn generate_media(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<GenerateMediaDto>,
) -> Result<Json<GenerateMediaRequest>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::generate_media(&dto, &claims, &state).await {
                Ok(generate_media_request) => Ok(Json(generate_media_request)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

#[debug_handler]
pub async fn import_media(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    multipart: Multipart,
) -> Result<Json<Vec<Media>>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => match service::import_media(multipart, &claims, &state).await {
            Ok(media) => Ok(Json(media)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

#[debug_handler]
pub async fn get_media(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetMediaFilterDto>,
) -> Result<Json<Vec<Media>>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::get_media(&dto, &claims, &state.pool).await {
                Ok(media) => Ok(Json(media)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

#[debug_handler]
pub async fn get_media_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Media>, ApiError> {
    match service::get_media_by_id_as_anonymous(&id, &state.pool).await {
        Ok(media) => Ok(Json(media)),
        Err(e) => Err(e),
    }
}

#[debug_handler]
pub async fn delete_media_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => service::delete_media_by_id(&id, &claims, &state).await,
        Err(e) => Err(e),
    }
}
