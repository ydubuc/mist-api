use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};

use crate::{
    app::util::time, auth::jwt::models::claims::Claims, devices::models::device::Device,
    users::models::user::User,
};

use super::config::JWT_EXP;

pub fn sign_jwt(user: &User, secret: &str, pepper: Option<&str>) -> String {
    let mut secret = secret.to_string();
    let iat = time::current_time_in_secs();
    let exp = iat + JWT_EXP;

    let claims = Claims {
        id: user.id.to_string(),
        roles: match &user.roles {
            Some(roles) => Some(roles.clone()),
            None => None,
        },
        iat,
        exp,
    };
    if let Some(pepper) = pepper {
        secret = [&secret, pepper].concat();
    }

    // FIXME: unsafe unwrap
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

pub fn sign_jwt_with_device(device: Device, secret: &str) -> String {
    let iat = time::current_time_in_secs();
    let exp = iat + JWT_EXP;

    let claims = Claims {
        id: device.user_id.to_string(),
        roles: match device.roles {
            Some(roles) => Some(roles),
            None => None,
        },
        iat,
        exp,
    };

    // FIXME: unsafe unwrap
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

pub fn decode_jwt(jwt: String, secret: &str, pepper: Option<&str>) -> Result<Claims, ErrorKind> {
    let mut secret = secret.to_string();

    if let Some(pepper) = pepper {
        secret = [&secret, pepper].concat();
    }

    let result = decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    );

    match result {
        Ok(data) => Ok(data.claims),
        Err(e) => Err(e.kind().to_owned()),
    }
}
