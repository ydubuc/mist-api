use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct DeleteAccountDto {
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
