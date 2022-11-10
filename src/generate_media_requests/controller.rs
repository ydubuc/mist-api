use axum::{
    extract::{Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use validator::Validate;

use crate::{app::models::api_error::ApiError, auth::jwt::models::claims::Claims, AppState};

use super::{
    dtos::get_generate_media_requests_filter_dto::GetGenerateMediaRequestsFilterDto,
    models::generate_media_request::GenerateMediaRequest, service,
};

pub async fn get_generate_media_requests(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetGenerateMediaRequestsFilterDto>,
) -> Result<Json<Vec<GenerateMediaRequest>>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match dto.validate() {
            Ok(_) => match service::get_generate_media_requests(&dto, &claims, &state.pool).await {
                Ok(generate_media_requests) => Ok(Json(generate_media_requests)),
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