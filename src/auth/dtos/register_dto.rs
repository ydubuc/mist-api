use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterDto {
    #[validate(length(
        min = 3,
        max = 24,
        message = "username must be between 3 and 24 characters."
    ))]
    #[validate(regex = "super::USERNAME_REGEX")]
    pub username: String,
    #[validate(email)]
    pub email: String,
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
