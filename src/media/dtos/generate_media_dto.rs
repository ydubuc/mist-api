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

    #[validate(range(min = 1, max = 8, message = "number must be between 1 and 8."))]
    pub number: u8,

    pub width: u16,

    pub height: u16,

    pub generator: String,

    // options
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(
        min = 1,
        max = 1000,
        message = "negative_prompt must be between 1 and 1000 characters."
    ))]
    pub negative_prompt: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1, max = 20, message = "cfg_scale must be between 1 and 20."))]
    pub cfg_scale: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(equal = 36, message = "input_media_id must be 36 characters."))]
    pub input_media_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish: Option<bool>,
}

impl GenerateMediaDto {
    pub fn sanitized(&self) -> Self {
        return Self {
            prompt: self.prompt.trim().replace("\n", " ").replace("\r", " "),
            number: self.number,
            width: self.width,
            height: self.height,
            generator: self.generator.to_string(),

            negative_prompt: match &self.negative_prompt {
                Some(negative_prompt) => {
                    Some(negative_prompt.trim().replace("\n", " ").replace("\r", " "))
                }
                None => None,
            },
            cfg_scale: self.cfg_scale,
            input_media_id: self.input_media_id.clone(),
            publish: self.publish,
        };
    }
}
