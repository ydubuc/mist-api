use axum::{
    extract::{Path, Query, State},
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
    dtos::{create_media_dto::CreateMediaDto, get_media_filter_dto::GetMediaFilterDto},
    models::media::Media,
    service,
};

pub async fn create_media(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<CreateMediaDto>,
) -> Result<Json<Vec<Media>>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match dto.validate() {
            Ok(_) => match service::create_media(&claims, &dto, &state.pool).await {
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

pub async fn get_media(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetMediaFilterDto>,
) -> Result<Json<Vec<Media>>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match dto.validate() {
            Ok(_) => match service::get_media(&claims, &dto, &state.pool).await {
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
        Ok(_) => match service::get_media_by_id(&id, &state.pool).await {
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
        Ok(claims) => return service::delete_media_by_id(&claims, &id, &state.pool).await,
        Err(e) => Err(e),
    }
}
