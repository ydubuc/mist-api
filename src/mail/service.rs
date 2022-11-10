use axum::http::StatusCode;
use lettre::{
    message::{header, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

use crate::app::{env::Envy, models::api_error::ApiError};

pub async fn send_mail(to: &str, subject: &str, body: &str, envy: &Envy) -> Result<(), ApiError> {
    let mail = lettre::Message::builder()
        .to(to.parse().unwrap())
        .from(envy.mail_user.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(String::from("Failed to display email.")),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(String::from(body)),
                ),
        )
        .unwrap();

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&envy.mail_host)
        .unwrap()
        .credentials(Credentials::new(
            envy.mail_user.to_string(),
            envy.mail_pass.to_string(),
        ))
        .build();

    match mailer.send(mail).await {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(%e);
            Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to send mail.".to_string(),
            })
        }
    }
}
