use axum::{
    extract::{Path, State},
    headers::{authorization::Bearer, Authorization},
    Json, TypedHeader,
};

use crate::{
    app::models::{api_error::ApiError, json_from_request::JsonFromRequest},
    auth::jwt::models::claims::Claims,
    AppState,
};

use super::{dtos::edit_device_dto::EditDeviceDto, models::device::Device, service};

pub async fn edit_device_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<EditDeviceDto>,
) -> Result<Json<Device>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => match service::edit_device_by_id(&id, &dto, &claims, &state.pool).await {
            Ok(device) => Ok(Json(device)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}
