use serde::Deserialize;
use validator::Validate;

// TODO: regex for password creation

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(length(
        min = 3,
        max = 24,
        message = "username must be between 3 and 24 characters."
    ))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub password: String,
}
