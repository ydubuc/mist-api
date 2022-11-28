use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GenerateMediaDto {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "prompt must be between 1 and 1000 characters."
    ))]
    pub prompt: String,
    #[validate(range(min = 1, max = 4, message = "number must be between 1 and 4."))]
    pub number: u8,
    pub width: u16,
    pub height: u16,
    pub generator: String,
}

impl GenerateMediaDto {
    pub fn sanitized(&self) -> Self {
        return Self {
            prompt: self.prompt.trim().replace("\n", "").replace("\r", ""),
            number: self.number,
            width: self.width,
            height: self.height,
            generator: self.generator.to_string(),
        };
    }
}
