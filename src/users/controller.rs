use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use validator::Validate;

use crate::{
    app::{models::api_error::ApiError, structs::json_from_request::JsonFromRequest},
    auth::jwt::models::claims::Claims,
    AppState,
};

use super::{
    dtos::{edit_user_dto::EditUserDto, get_users_filter_dto::GetUsersFilterDto},
    models::user::User,
    service,
};

pub async fn get_users(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetUsersFilterDto>,
) -> Result<Json<Vec<User>>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::get_users(&dto, &claims, &state.pool).await {
                Ok(users) => Ok(Json(users)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn get_user_from_request(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<User>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => match service::get_user_by_id(&claims.id, &claims, &state.pool).await {
            Ok(user) => Ok(Json(user)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<User>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => match service::get_user_by_id(&id, &claims, &state.pool).await {
            Ok(user) => Ok(Json(user)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub async fn edit_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<EditUserDto>,
) -> Result<Json<User>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::edit_user_by_id(&id, &dto, &claims, &state.pool).await {
                Ok(user) => Ok(Json(user)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}
