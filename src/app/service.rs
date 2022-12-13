use std::sync::Arc;

use axum::headers::{authorization::Bearer, Authorization};
use reqwest::{header, StatusCode};
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
    let dalle_status = api_state.dalle_status.read().await.to_string();
    let labml_status = api_state.labml_status.read().await.to_string();
    let mist_stability_status = api_state.mist_stability_status.read().await.to_string();
    let stable_horde_status = api_state.stable_horde_status.read().await.to_string();

    return json!({
        "api_status": api_status,
        "dalle_status": dalle_status,
        "labml_status": labml_status,
        "mist_stability_status": mist_stability_status,
        "stable_horde_status": stable_horde_status,
    });
}

pub async fn edit_api_state(
    dto: &EditApiStatusDto,
    claims: &Claims,
    authorization: &Authorization<Bearer>,
    state: &Arc<AppState>,
) -> Result<Value, ApiError> {
    let Some(roles) = &claims.roles
    else {
        return Err(DefaultApiError::PermissionDenied.value());
    };

    if !roles.contains(&Roles::ADMIN.to_string()) {
        return Err(DefaultApiError::PermissionDenied.value());
    }

    tracing::debug!("edit_api_state");

    let api_state = &state.api_state;

    if let Some(api_status) = &dto.api_status {
        let mut current_status = api_state.api_status.write().await;
        *current_status = api_status.to_string();

        drop(current_status);
    }
    if let Some(dalle_status) = &dto.dalle_status {
        let mut current_status = api_state.dalle_status.write().await;
        *current_status = dalle_status.to_string();

        drop(current_status);
    }
    if let Some(labml_status) = &dto.labml_status {
        let mut current_status = api_state.labml_status.write().await;
        *current_status = labml_status.to_string();

        drop(current_status);
    }
    if let Some(mist_stability_status) = &dto.mist_stability_status {
        let mut current_status = api_state.mist_stability_status.write().await;
        *current_status = mist_stability_status.to_string();

        drop(current_status);
    }
    if let Some(stable_horde_status) = &dto.stable_horde_status {
        let mut current_status = api_state.stable_horde_status.write().await;
        *current_status = stable_horde_status.to_string();

        drop(current_status);
    }

    if dto.send_signal {
        match state.envy.app_env.as_str() {
            "production" => {
                let urls: [&str; 3] = [
                    "https://mist-api-1-production.up.railway.app/status",
                    "https://mist-api-2-production.up.railway.app/status",
                    "https://mist-api-3-production.up.railway.app/status",
                ];

                for url in urls {
                    spawn_send_signal_edit_api_state(url, dto, authorization);
                }
            }
            "development" => {
                let urls: [&str; 3] = [
                    "https://mist-api-1-development.up.railway.app/status",
                    "https://mist-api-2-development.up.railway.app/status",
                    "https://mist-api-3-development.up.railway.app/status",
                ];

                for url in urls {
                    spawn_send_signal_edit_api_state(url, dto, authorization);
                }
            }
            _ => {
                // not sending signal
            }
        }
    }

    return Ok(get_api_state(state).await);
}

fn spawn_send_signal_edit_api_state(
    url: &str,
    dto: &EditApiStatusDto,
    authorization: &Authorization<Bearer>,
) {
    let url = url.to_string();
    let dto = EditApiStatusDto {
        api_status: dto.api_status.clone(),
        dalle_status: dto.dalle_status.clone(),
        labml_status: dto.labml_status.clone(),
        mist_stability_status: dto.mist_stability_status.clone(),
        stable_horde_status: dto.stable_horde_status.clone(),
        send_signal: false,
    };
    let authorization = authorization.clone();

    tokio::spawn(async move {
        let mut headers = header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert(
            "Authorization",
            ["Bearer ", authorization.token()].concat().parse().unwrap(),
        );

        let client = reqwest::Client::new();
        let result = client.patch(&url).headers(headers).json(&dto).send().await;

        match result {
            Ok(res) => match res.text().await {
                Ok(text) => tracing::debug!("{}: {:?}", url, text),
                Err(e) => {
                    tracing::warn!("spawn_send_signal_edit_api_state (2): {:?}", e)
                }
            },
            Err(e) => {
                tracing::warn!("spawn_send_signal_edit_api_state (3) ({}): {:?}", url, e);
            }
        }
    });
}
