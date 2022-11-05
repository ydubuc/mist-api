use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct RefreshDeviceDto {
    #[validate(length(equal = 36, message = "device_id must be 36 characters."))]
    pub device_id: String,
    #[validate(length(equal = 36, message = "user_id must be 36 characters."))]
    pub user_id: String,
    #[validate(length(equal = 36, message = "refresh_token must be 36 characters."))]
    pub refresh_token: String,
}
