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

pub fn sign_jwt(uid: &str) -> String {
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
    let secret = env::var(Env::JWT_SECRET).expect("environment: JWT_SECRET missing");
    let jwt = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();

    jwt
}

pub fn decode_jwt(jwt: String) -> Result<Claims, ErrorKind> {
    let secret = env::var(Env::JWT_SECRET).expect("environment: JWT_SECRET missing");
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
