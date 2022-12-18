use std::sync::Arc;

use serde_json::{json, Value};

use crate::{
    auth::jwt::{enums::roles::Roles, models::claims::Claims},
    AppState,
};

use super::{
    dtos::edit_api_status_dto::EditApiStatusDto, errors::DefaultApiError,
    models::api_error::ApiError,
};

pub async fn get_api_state(state: &Arc<AppState>) -> Value {
    let api_state = &state.api_state;

    let api_status = api_state.api_status.read().await.to_string();

    return json!({
        "api_status": api_status,
    });
}

pub async fn edit_api_state(
    dto: &EditApiStatusDto,
    claims: &Claims,
    state: &Arc<AppState>,
) -> Result<Value, ApiError> {
    let Some(roles) = &claims.roles
    else {
        return Err(DefaultApiError::PermissionDenied.value());
    };

    if !roles.contains(&Roles::ADMIN.to_string()) {
        return Err(DefaultApiError::PermissionDenied.value());
    }

    let api_state = &state.api_state;

    if let Some(api_status) = &dto.api_status {
        let mut current_status = api_state.api_status.write().await;
        *current_status = api_status.to_string();

        drop(current_status);
    }

    return Ok(get_api_state(state).await);
}
