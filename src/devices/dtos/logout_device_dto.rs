use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct LogoutDeviceDto {
    pub device_ids: Vec<String>,
}
