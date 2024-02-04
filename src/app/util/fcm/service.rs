use std::sync::Arc;

use tokio::sync::RwLock;

use super::client::fcm_client::FcmClient;

pub async fn check_token(fcm_client: &Arc<RwLock<FcmClient>>) {
    let _fcm_client = fcm_client.read().await;

    if _fcm_client.token_time.elapsed().as_secs() > 3600 {
        drop(_fcm_client);

        let mut fcm_client = fcm_client.write().await;

        match fcm_client.check_token().await {
            Ok(_) => {}
            Err(e) => tracing::error!("check_token: {:?}", e),
        }
    }
}
