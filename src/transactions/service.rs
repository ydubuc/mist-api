use reqwest::StatusCode;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    app::{models::api_error::ApiError, util::time},
    transactions::handlers,
    AppState,
};

use super::structs::revenue_cat_webbook::RevenueCatWebhook;

pub async fn handle_webhook(webhook: RevenueCatWebhook, state: &AppState) -> Result<(), ApiError> {
    println!("handling webhook {:?}", webhook);

    let Some(event_type) = webhook.event.get("type")
    else {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: "Event has no type.".to_string()
        });
    };

    let event = event_type.as_str().unwrap();

    match event {
        "NON_RENEWING_PURCHASE" => handlers::non_renewing_purchase::handle(webhook, state).await,
        _ => {
            tracing::error!("Not handling webhook event type: {}", event);
            return Ok(());
        }
    }
}

pub fn retrieve_user_id(
    app_user_id: &str,
    original_app_user_id: &str,
    aliases: &Vec<String>,
) -> Option<String> {
    let anonymous = "$RCAnonymousID:";

    if !app_user_id.starts_with(anonymous) {
        return Some(app_user_id.to_string());
    }

    if !original_app_user_id.starts_with(anonymous) {
        return Some(original_app_user_id.to_string());
    }

    let user_ids: Vec<String> = aliases
        .clone()
        .into_iter()
        .filter(|alias| !alias.starts_with(anonymous))
        .collect();

    if user_ids.len() > 0 {
        return Some(user_ids.first().unwrap().to_string());
    }

    None
}

pub async fn create_transaction(
    webhook: RevenueCatWebhook,
    user_id: &str,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), ApiError> {
    println!("create_transaction");

    let sql = r#"
    INSERT INTO transactions (id, user_id, data, created_at)
    VALUES ($1, $2, $3, $4)
    "#;

    let sqlx_result = sqlx::query(&sql)
        .bind(Uuid::new_v4().to_string())
        .bind(user_id)
        .bind(webhook.event)
        .bind(time::current_time_in_secs() as i64)
        .execute(&mut *tx)
        .await;

    match sqlx_result {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(%e);
            return Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to create transaction.".to_string(),
            });
        }
    }
}
