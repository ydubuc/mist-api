use crate::app::models::api_error::ApiError;

use super::structs::revenue_cat_webbook::RevenueCatWebhook;

pub async fn handle_webhook(webhook: RevenueCatWebhook) -> Result<(), ApiError> {
    println!("handling webhook {:?}", webhook);

    Ok(())
}
