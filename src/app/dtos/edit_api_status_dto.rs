use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::app::enums::api_status::ApiStatus;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct EditApiStatusDto {
    #[validate(custom(function = "validate_status"))]
    pub api_status: Option<String>,
    pub send_signal: bool,
}

fn validate_status(value: &str) -> Result<(), ValidationError> {
    if value != ApiStatus::Online.value() && value != ApiStatus::Maintenance.value() {
        return Err(ValidationError::new("validate_status"));
    }

    return Ok(());
}
