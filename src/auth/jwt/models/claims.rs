use axum::{
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
};
use jsonwebtoken::errors::ErrorKind;
use serde::{Deserialize, Serialize};

use crate::{
    app::models::api_error::ApiError,
    auth::jwt::{enums::roles::Roles, util::decode_jwt},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    pub iat: u64,
    pub exp: u64,
}

impl Claims {
    pub fn from_header(
        authorization: Authorization<Bearer>,
        secret: &str,
    ) -> Result<Self, ApiError> {
        match decode_jwt(authorization.0.token().to_string(), secret, None) {
            Ok(claims) => return Ok(claims),
            Err(e) => match e {
                ErrorKind::ExpiredSignature => {
                    return Err(ApiError {
                        code: StatusCode::UNAUTHORIZED,
                        message: "Token expired.".to_string(),
                    });
                }
                _ => {
                    return Err(ApiError {
                        code: StatusCode::UNAUTHORIZED,
                        message: "Invalid token.".to_string(),
                    });
                }
            },
        }
    }

    pub fn is_mod(&self) -> bool {
        let Some(roles) = &self.roles
        else { return false; };

        return roles.contains(&Roles::MODERATOR.to_string());
    }
}
