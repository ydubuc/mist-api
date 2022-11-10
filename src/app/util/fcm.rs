pub async fn send_notification(
    messaging_token: String,
    title: String,
    body: String,
    fcm_api_key: String,
) -> Result<(), String> {
    let client = fcm::Client::new();

    let mut builder = fcm::NotificationBuilder::new();
    builder.title(&title);
    builder.body(&body);

    let notification = builder.finalize();

    let mut message_builder = fcm::MessageBuilder::new(&fcm_api_key, &messaging_token);
    message_builder.notification(notification);

    match client.send(message_builder.finalize()).await {
        Ok(res) => Ok(()),
        Err(e) => {
            tracing::error!(%e);

            match e {
                fcm::Error::Unauthorized => Ok(()),
                fcm::Error::InvalidMessage(_) => Ok(()),
                fcm::Error::ServerError(_) => Err(messaging_token.to_string()),
            }
        }
    }
}
