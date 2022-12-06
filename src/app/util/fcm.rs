pub async fn send_notification(
    messaging_token: String,
    title: String,
    body: String,
    click_action: Option<String>,
    fcm_api_key: String,
) -> Result<(), String> {
    let client = fcm::Client::new();

    let mut builder = fcm::NotificationBuilder::new();
    builder.title(&title);
    builder.body(&body);
    if let Some(click_action) = &click_action {
        builder.click_action(click_action);
    }
    builder.sound("default");

    let notification = builder.finalize();

    let mut message_builder = fcm::MessageBuilder::new(&fcm_api_key, &messaging_token);
    message_builder.notification(notification);

    match client.send(message_builder.finalize()).await {
        Ok(_) => Ok(()),
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
