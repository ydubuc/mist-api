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
    dtos::{edit_user_dto::EditUserDto, get_users_filter_dto::GetUsersFilterDto},
    models::user::User,
    service,
};

pub async fn get_users(
    State(state): State<AppState>,
    TypedHeader(_authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetUsersFilterDto>,
) -> Result<Json<Vec<User>>, ApiError> {
    match dto.validate() {
        Ok(_) => match service::get_users(&dto, &state.pool).await {
            Ok(users) => Ok(Json(users)),
            Err(e) => Err(e),
        },
        Err(e) => Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        }),
    }
}

pub async fn get_user_from_request(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<User>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match service::get_user_by_id(&claims.id, &state.pool).await {
            Ok(user) => Ok(Json(user)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(_authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<User>, ApiError> {
    match service::get_user_by_id(&id, &state.pool).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => Err(e),
    }
}

pub async fn edit_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<EditUserDto>,
) -> Result<Json<User>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match dto.validate() {
            Ok(_) => match service::edit_user_by_id(&claims, &id, &dto, &state.pool).await {
                Ok(user) => Ok(Json(user)),
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

pub async fn delete_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => return service::delete_user_by_id(&claims, &id, &state.pool).await,
        Err(e) => Err(e),
    }
}
