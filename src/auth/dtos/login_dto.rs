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
    #[validate(regex(path = "super::USERNAME_REGEX"))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(
        length(
            min = 8,
            max = 512,
            message = "password must be between at least 8 characters."
        ),
        custom = "super::validate_password"
    )]
    pub password: String,
}
