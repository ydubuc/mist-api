use fcm::{response, ErrorReason, FcmError, FcmResponse, Message, RetryAfter};
use reqwest::{
    header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, RETRY_AFTER},
    Body, StatusCode,
};

pub async fn send_notification(
    messaging_token: String,
    title: String,
    body: String,
    click_action: Option<String>,
    fcm_api_key: String,
    fcm_client: FcmClient,
) -> Result<(), String> {
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

    match fcm_client.send(message_builder.finalize()).await {
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

/// An async client for sending the notification payload.
#[derive(Clone, Debug)]
pub struct FcmClient {
    http_client: reqwest::Client,
}

impl Default for FcmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl FcmClient {
    /// Get a new instance of Client.
    pub fn new() -> FcmClient {
        let http_client = reqwest::ClientBuilder::new()
            .pool_max_idle_per_host(std::usize::MAX)
            .build()
            .unwrap();

        FcmClient { http_client }
    }

    /// Try sending a `Message` to FCM.
    pub async fn send(&self, message: Message<'_>) -> Result<FcmResponse, FcmError> {
        let payload = serde_json::to_vec(&message.body).unwrap();

        let request = self
            .http_client
            .post("https://fcm.googleapis.com/fcm/send")
            .header(CONTENT_TYPE, "application/json")
            .header(
                CONTENT_LENGTH,
                format!("{}", payload.len() as u64).as_bytes(),
            )
            .header(AUTHORIZATION, format!("key={}", message.api_key).as_bytes())
            .body(Body::from(payload))
            .build()?;
        let response = self.http_client.execute(request).await?;

        let response_status = response.status();

        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|ra| ra.to_str().ok())
            .and_then(|ra| ra.parse::<RetryAfter>().ok());

        match response_status {
            StatusCode::OK => {
                let fcm_response: FcmResponse = response.json().await.unwrap();

                match fcm_response.error {
                    Some(ErrorReason::Unavailable) => {
                        Err(response::FcmError::ServerError(retry_after))
                    }
                    Some(ErrorReason::InternalServerError) => {
                        Err(response::FcmError::ServerError(retry_after))
                    }
                    _ => Ok(fcm_response),
                }
            }
            StatusCode::UNAUTHORIZED => Err(response::FcmError::Unauthorized),
            StatusCode::BAD_REQUEST => Err(response::FcmError::InvalidMessage(
                "Bad Request".to_string(),
            )),
            status if status.is_server_error() => Err(response::FcmError::ServerError(retry_after)),
            _ => Err(response::FcmError::InvalidMessage(
                "Unknown Error".to_string(),
            )),
        }
    }
}
