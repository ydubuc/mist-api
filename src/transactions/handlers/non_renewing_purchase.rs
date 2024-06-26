use reqwest::StatusCode;

use crate::{
    app::models::api_error::ApiError,
    transactions::{
        handlers::errors::HandlersApiError,
        service,
        structs::{
            revenuecat_event_non_renewing::RevenueCatWebhookEventNonRenewing,
            revenuecat_webbook::RevenueCatWebhook,
        },
        INK_LARGE_AMOUNT, INK_MEDIUM_AMOUNT, INK_MEGA_AMOUNT, INK_SMALL_AMOUNT,
    },
    users::{self, util::ink::dtos::edit_user_ink_dto::EditUserInkDto},
    AppState,
};

pub async fn handle(webhook: RevenueCatWebhook, state: &AppState) -> Result<(), ApiError> {
    let event: RevenueCatWebhookEventNonRenewing =
        serde_json::from_value(webhook.clone().event).unwrap();

    let Some(user_id) = service::retrieve_user_id(
        &event.app_user_id,
        &event.original_app_user_id,
        &event.aliases,
    ) else {
        tracing::error!(
            "WEBHOOK ERROR<handle_non_renewing_purchase>: NO USER_ID FOUND ({})",
            event.id
        );
        return Err(ApiError {
            code: StatusCode::NOT_FOUND,
            message: "Failed to get user_id from event.".to_string(),
        });
    };

    let amount = match event.product_id.as_ref() {
        "com.greenknightlabs.mist.ios.ink_small.111622" => INK_SMALL_AMOUNT,
        "com.greenknightlabs.mist.ios.ink_medium.111622" => INK_MEDIUM_AMOUNT,
        "com.greenknightlabs.mist.ios.ink_large.111622" => INK_LARGE_AMOUNT,
        "com.greenknightlabs.mist.ios.ink_mega.111622" => INK_MEGA_AMOUNT,
        _ => {
            tracing::error!("product_id not implemented: {}", event.product_id);
            return Err(HandlersApiError::ProductNotImplemented.value());
        }
    };

    let Ok(mut tx) = state.pool.begin().await else {
        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to begin pool transaction.".to_string(),
        });
    };

    let edit_user_ink_dto = EditUserInkDto {
        ink_increase: Some(amount),
        ink_decrease: None,
        ink_sum_increase: Some(amount),
        ink_sum_decrease: None,
        ink_pending_increase: None,
        ink_pending_decrease: None,
    };

    let edit_user_ink_by_id_result =
        users::util::ink::ink::edit_user_ink_by_id(&user_id, &edit_user_ink_dto, &mut tx).await;

    if edit_user_ink_by_id_result.is_err() {
        let rollback_result = tx.rollback().await;

        if let Some(e) = rollback_result.err() {
            tracing::error!(
                "handle_non_renewing_purchase failed to roll back edit_user_ink_by_id_result: {:?}",
                e
            );
        } else {
            tracing::warn!("handle_non_renewing_purchase rolled back edit_user_ink_by_id_result")
        }

        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to edit user ink.".to_string(),
        });
    }

    let create_transaction_result =
        service::create_transaction(&event.id, webhook, &user_id, &mut tx).await;

    if create_transaction_result.is_err() {
        let rollback_result = tx.rollback().await;

        if let Some(e) = rollback_result.err() {
            tracing::error!(
                "handle_non_renewing_purchase failed to roll back create_transaction_result: {:?}",
                e
            );
        } else {
            tracing::warn!("handle_non_renewing_purchase rolled back create_transaction_result");
        }

        return Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Failed to save transaction.".to_string(),
        });
    }

    match tx.commit().await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("handle_non_renewing_purchase failed to commit tx: {:?}", e);
            return Err(HandlersApiError::TransactionError.value());
        }
    }
}
