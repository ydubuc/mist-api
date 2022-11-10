use axum::http::StatusCode;
use lettre::{
    message::{header, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

use crate::app::{env::Env, models::api_error::ApiError};

pub async fn send_mail(to: &str, subject: &str, body: &str) -> Result<(), ApiError> {
    let mail_host = std::env::var(Env::MAIL_HOST).unwrap();
    let mail_user = std::env::var(Env::MAIL_USER).unwrap();
    let mail_pass = std::env::var(Env::MAIL_PASS).unwrap();

    let mail = lettre::Message::builder()
        .to(to.parse().unwrap())
        .from(mail_user.parse().unwrap())
        .subject(subject)
        // .body(String::from(body))
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

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&mail_host)
        .unwrap()
        .credentials(Credentials::new(mail_user, mail_pass))
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
