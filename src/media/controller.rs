use axum::{
    extract::{Multipart, Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use validator::Validate;

use crate::{
    app::models::{api_error::ApiError, json_from_request::JsonFromRequest},
    auth::jwt::models::claims::Claims,
    AppState,
};

use super::{
    dtos::{generate_media_dto::GenerateMediaDto, get_media_filter_dto::GetMediaFilterDto},
    models::media::Media,
    service,
};

pub async fn generate_media(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<GenerateMediaDto>,
) -> Result<Json<Vec<Media>>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match dto.validate() {
            Ok(_) => match service::generate_media(&dto, &claims, &state.pool, &state.b2).await {
                Ok(media) => Ok(Json(media)),
                Err(e) => Err(e),
            },
            Err(e) => Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: e.to_string(),
            }),
        },
        Err(e) => Err(e),
    }
}

pub async fn import_media(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    multipart: Multipart,
) -> Result<Json<Vec<Media>>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match service::import_media(multipart, &claims, &state.pool, &state.b2).await
        {
            Ok(media) => Ok(Json(media)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub async fn get_media(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetMediaFilterDto>,
) -> Result<Json<Vec<Media>>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match dto.validate() {
            Ok(_) => match service::get_media(&dto, &claims, &state.pool).await {
                Ok(media) => Ok(Json(media)),
                Err(e) => Err(e),
            },
            Err(e) => Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: e.to_string(),
            }),
        },
        Err(e) => Err(e),
    }
}

pub async fn get_media_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Media>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match service::get_media_by_id(&id, &claims, &state.pool).await {
            Ok(media) => Ok(Json(media)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub async fn delete_media_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => {
            return service::delete_media_by_id(&id, &claims, &state.pool, &state.b2).await
        }
        Err(e) => Err(e),
    }
}
