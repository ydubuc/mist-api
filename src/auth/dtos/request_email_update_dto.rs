use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RequestEmailUpdateDto {
    #[validate(email)]
    pub email: String,
}
