use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};

use crate::{app::util::time, auth::jwt::models::claims::Claims};

use super::config::JWT_EXP;

pub fn sign_jwt(id: &str, secret: &str, pepper: Option<&str>) -> String {
    let mut secret = secret.to_string();
    let iat = time::current_time_in_secs();
    let exp = iat + JWT_EXP;

    let claims = Claims {
        id: id.to_string(),
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
