use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct EditEmailDto {
    #[validate(email)]
    pub email: String,
}
