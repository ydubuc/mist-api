use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};

use crate::{app::env::Env, auth::jwt::models::claims::Claims};

use super::config::JWT_EXP;

// FIXME: unsafe unwraps

pub fn sign_jwt(uid: &str, pepper: Option<&str>) -> String {
    let iat = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let exp = iat + JWT_EXP;

    let claims = Claims {
        id: uid.to_string(),
        iat,
        exp,
    };
    let mut secret = env::var(Env::JWT_SECRET).unwrap();
    if let Some(pepper) = pepper {
        secret = secret + pepper
    }

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

pub fn decode_jwt(jwt: String, pepper: Option<&str>) -> Result<Claims, ErrorKind> {
    let mut secret = env::var(Env::JWT_SECRET).unwrap();
    if let Some(pepper) = pepper {
        secret = secret + pepper;
    }

    let result = decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(&secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    );

    match result {
        Ok(data) => Ok(data.claims),
        Err(e) => Err(e.kind().to_owned()),
    }
}
