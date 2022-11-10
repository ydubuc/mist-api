use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RequestPasswordUpdateDto {
    #[validate(email)]
    pub email: String,
}
