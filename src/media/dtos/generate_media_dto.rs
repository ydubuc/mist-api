use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    app::models::api_error::ApiError,
    media::{
        apis::{dalle, mist, stable_horde},
        enums::{media_generator::MediaGenerator, media_model::MediaModel},
    },
};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
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
    pub fn default_model(&self) -> &str {
        match self.generator.as_ref() {
            MediaGenerator::MIST => MediaModel::STABLE_DIFFUSION_1_5,
            MediaGenerator::STABLE_HORDE => MediaModel::STABLE_DIFFUSION_1_5,
            MediaGenerator::DALLE => MediaModel::DALLE,
            _ => panic!("default_model for generator not implemented."),
        }
    }

    pub fn default_cfg_scale() -> u8 {
        return 8;
    }

    pub fn sanitized(&self) -> Self {
        return Self {
            prompt: self.prompt.trim().replace("\n", " ").replace("\r", " "),
            number: self.number,
            width: self.width,
            height: self.height,
            generator: self.generator.to_string(),
            model: Some(
                self.model
                    .clone()
                    .unwrap_or(self.default_model().to_string()),
            ),
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

    pub fn is_valid(&self) -> Result<(), ApiError> {
        let model = self
            .model
            .clone()
            .unwrap_or(self.default_model().to_string());

        let is_valid_generator: bool;
        let is_valid_model: bool;
        let is_valid_size: bool;
        let is_valid_number: bool;

        match self.generator.as_ref() {
            MediaGenerator::MIST => {
                is_valid_generator = true;
                is_valid_model = mist::service::is_valid_model(&model);
                is_valid_size = mist::service::is_valid_size(&self.width, &self.height, &model);
                is_valid_number = mist::service::is_valid_number(self.number, &model);
            }
            MediaGenerator::STABLE_HORDE => {
                is_valid_generator = true;
                is_valid_model = stable_horde::service::is_valid_model(&model);
                is_valid_size = stable_horde::service::is_valid_size(&self.width, &self.height);
                is_valid_number = stable_horde::service::is_valid_number(self.number);
            }
            MediaGenerator::DALLE => {
                is_valid_generator = true;
                is_valid_model = dalle::service::is_valid_model(&model);
                is_valid_size = dalle::service::is_valid_size(&self.width, &self.height);
                is_valid_number = dalle::service::is_valid_number(self.number);
            }
            _ => {
                is_valid_generator = false;
                is_valid_model = false;
                is_valid_size = false;
                is_valid_number = false;
            }
        }

        if !is_valid_generator {
            return Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: "This generator is not supported.".to_string(),
            });
        }
        if !is_valid_model {
            return Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: "This generator does not support this model.".to_string(),
            });
        }
        if !is_valid_size {
            return Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: "This generator does not support this size.".to_string(),
            });
        }
        if !is_valid_number {
            return Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: "This generator does not support this number.".to_string(),
            });
        }

        Ok(())
    }
}
