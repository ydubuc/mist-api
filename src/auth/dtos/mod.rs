use regex::Regex;
use validator::ValidationError;

pub mod edit_password_dto;
pub mod login_dto;
pub mod register_dto;
pub mod request_email_update_dto;
pub mod request_password_update_dto;

lazy_static! {
    pub static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_.-]{3,24}$").unwrap();
}

lazy_static! {
    pub static ref PASSWORD_REGEX: Regex =
        Regex::new(r"^(.{0,7}|[^0-9]*|[^A-Z]*|[^a-z]*|[a-zA-Z0-9]*)$").unwrap();
}

pub fn validate_password(value: &str) -> Result<(), ValidationError> {
    match PASSWORD_REGEX.is_match(value) {
        true => Err(ValidationError::new("password_validation")),
        false => Ok(()),
    }
}
