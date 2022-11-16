use futures::future::BoxFuture;
use reqwest::StatusCode;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    app::{models::api_error::ApiError, util::time},
    transactions::structs::revenue_cat_event_non_renewing::RevenueCatWebhookEventNonRenewing,
    users::models::user::User,
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
        "NON_RENEWING_PURCHASE" => return handle_non_renewing_event(webhook, state).await,
        _ => {
            println!("Not handling webhook event type: {}", event);
            return Ok(());
        }
    }
}

async fn handle_non_renewing_event(
    webhook: RevenueCatWebhook,
    state: &AppState,
) -> Result<(), ApiError> {
    println!("handling non renewing event");

    let event: RevenueCatWebhookEventNonRenewing =
        serde_json::from_value(webhook.clone().event).unwrap();

    let Some(user_id) = retrieve_user_id(
        &event.app_user_id,
        &event.original_app_user_id,
        &event.aliases,
    ) else {
        tracing::error!("WEBHOOK ERROR: handle_non_renewing_event NO USER_ID FOUND.");
        return Err(ApiError {
            code: StatusCode::NOT_FOUND,
            message: "Failed to get user_id from event.".to_string()
        });
    };

    match event.product_id.as_ref() {
        "com.greenknightlabs.mist.ios.ink_small.111622" => {
            let Ok(mut tx) = state.pool.begin().await
            else {
                return Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "Failed to being pool transaction.".to_string(),
                });
            };

            let result_1 = update_user_ink_by_id(&user_id, 50, &mut tx).await;
            println!("complete update_user_ink_by_id");
            let result_2 = create_transaction(webhook, &mut tx).await;
            println!("complete update_user_ink_by_id");

            match tx.commit().await {
                Ok(_) => {
                    println!("tx ok");
                    return if result_1.is_ok() && result_2.is_ok() {
                        Ok(())
                    } else {
                        println!("database transaction did not complete");
                        return Err(ApiError {
                            code: StatusCode::INTERNAL_SERVER_ERROR,
                            message: "Database transaction did not go through.".to_string(),
                        });
                    };
                }
                Err(e) => {
                    tracing::error!(%e);
                    return Err(ApiError {
                        code: StatusCode::INTERNAL_SERVER_ERROR,
                        message: "Database transaction failed.".to_string(),
                    });
                }
            }
        }
        _ => {
            tracing::error!("Not implemented product_id: {}", event.product_id);
            return Err(ApiError {
                code: StatusCode::NOT_IMPLEMENTED,
                message: "Product Id not implemented.".to_string(),
            });
        }
    }
}

fn retrieve_user_id(
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

async fn update_user_ink_by_id(
    id: &str,
    amount: i32,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), ApiError> {
    println!("update_user_ink_by_id");

    let sql = match amount >= 0 {
        true => {
            r#"
            UPDATE users
            SET ink = ink + $1
            WHERE id = $2
            "#
        }
        false => {
            r#"
            UPDATE users
            SET ink = ink - $1
            WHERE id = $2
            "#
        }
    };

    match sqlx::query(&sql)
        .bind(amount)
        .bind(id)
        .execute(&mut *tx)
        .await
    {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => {
                tracing::error!("WEBHOOK ERROR: update_user_ink_by_id NO ROWS AFFECTED");
                tracing::error!(%id);

                return Err(ApiError {
                    code: StatusCode::NOT_FOUND,
                    message: "User not found.".to_string(),
                });
            }
        },
        Err(e) => {
            tracing::error!(%e);
            return Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to update user ink.".to_string(),
            });
        }
    }
}

async fn create_transaction(
    webhook: RevenueCatWebhook,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), ApiError> {
    println!("create_transaction");

    let sql = r#"
    INSERT INTO transactions (id, data, created_at)
    VALUES ($1, $2, $3)
    "#;

    let sqlx_result = sqlx::query(&sql)
        .bind(Uuid::new_v4().to_string())
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
