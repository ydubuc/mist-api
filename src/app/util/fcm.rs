use crate::app::{env::Env, models::app_error::AppError};

pub async fn send_notification(
    messaging_token: String,
    title: String,
    body: String,
) -> Result<(), String> {
    let api_key = std::env::var(Env::FCM_API_KEY).unwrap();
    let client = fcm::Client::new();

    let mut builder = fcm::NotificationBuilder::new();
    builder.title(&title);
    builder.body(&body);

    let notification = builder.finalize();

    let mut message_builder = fcm::MessageBuilder::new(&api_key, &messaging_token);
    message_builder.notification(notification);

    match client.send(message_builder.finalize()).await {
        Ok(res) => Ok(()),
        Err(e) => {
            tracing::error!(%e);

            match e {
                fcm::Error::Unauthorized => Ok(()),
                fcm::Error::InvalidMessage(_) => Ok(()),
                fcm::Error::ServerError(_) => Err(messaging_token),
            }
        }
    }
}
