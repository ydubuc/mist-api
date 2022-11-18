use reqwest::StatusCode;

use crate::{
    app::models::api_error::ApiError,
    media::{self, util::ink::dtos::edit_user_dto::EditUserInkDto},
    transactions::{
        handlers::{
            errors::HandlersApiError, INK_LARGE_AMOUNT, INK_MEDIUM_AMOUNT, INK_SMALL_AMOUNT,
        },
        service,
        structs::{
            revenue_cat_event_non_renewing::RevenueCatWebhookEventNonRenewing,
            revenue_cat_webbook::RevenueCatWebhook,
        },
    },
    AppState,
};

pub async fn handle(webhook: RevenueCatWebhook, state: &AppState) -> Result<(), ApiError> {
    println!("handling non renewing purchase");

    let event: RevenueCatWebhookEventNonRenewing =
        serde_json::from_value(webhook.clone().event).unwrap();

    let Some(user_id) = service::retrieve_user_id(
        &event.app_user_id,
        &event.original_app_user_id,
        &event.aliases,
    ) else {
        tracing::error!("WEBHOOK ERROR<handle_non_renewing_purchase>: NO USER_ID FOUND ({})", event.id);
        return Err(ApiError {
            code: StatusCode::NOT_FOUND,
            message: "Failed to get user_id from event.".to_string()
        });
    };

    let amount = match event.product_id.as_ref() {
        "com.greenknightlabs.mist.ios.ink_small.111622" => INK_SMALL_AMOUNT,
        "com.greenknightlabs.mist.ios.ink_medium.111622" => INK_MEDIUM_AMOUNT,
        "com.greenknightlabs.mist.ios.ink_large.111622" => INK_LARGE_AMOUNT,
        _ => {
            tracing::error!("Not implemented product_id: {}", event.product_id);
            return Err(HandlersApiError::ProductNotImplemented.value());
        }
    };

    let Ok(mut tx) = state.pool.begin().await
    else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to begin pool transaction.".to_string(),
        });
    };

    let edit_user_ink_dto = EditUserInkDto {
        ink_increase: Some(amount),
        ink_decrease: None,
        ink_pending_increase: None,
        ink_pending_decrease: None,
    };

    let result_1 =
        media::util::ink::ink::edit_user_ink_by_id(&user_id, &edit_user_ink_dto, &mut tx).await;
    println!("complete update_user_ink_by_id");

    let result_2 = service::create_transaction(webhook, &user_id, &mut tx).await;
    println!("complete update_user_ink_by_id");

    match tx.commit().await {
        Ok(_) => {
            return if result_1.is_ok() && result_2.is_ok() {
                println!("tx ok");
                Ok(())
            } else {
                tracing::error!("database transaction did not complete");
                return Err(HandlersApiError::TransactionFailure.value());
            };
        }
        Err(e) => {
            tracing::error!(%e);
            return Err(HandlersApiError::TransactionError.value());
        }
    }
}
