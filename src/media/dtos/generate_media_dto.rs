use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateMediaDto {
    #[validate(length(
        min = 1,
        max = 400,
        message = "promt must be between 1 and 400 characters."
    ))]
    pub prompt: String,
    #[validate(range(min = 1, max = 4, message = "number must be between 1 and 4."))]
    pub number: u8,
    pub width: u16,
    pub height: u16,
}
