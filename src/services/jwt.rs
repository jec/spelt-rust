use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::error::Error;

pub const JWT_TTL_SECONDS: u64 = 300;

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    sub: String,
    iat: u64,
    nbf: u64,
    exp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtValidator {
    header: String,
    claims: JwtClaims,
}

pub fn create_jwt(session_uuid: &String) -> Result<String, Error> {
    let now: u64 = Utc::now().timestamp() as u64;
    let claims = JwtClaims {
        sub: session_uuid.into(),
        iat: now,
        nbf: now,
        exp: now + JWT_TTL_SECONDS
    };

    // The path to the PEM file is relative to this source file and is loaded at
    // compile time.
    Ok(encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(include_bytes!("../../config/pkey.pem"))?
    )?)
}

pub fn validate_jwt(jwt: String) -> Result<(), Error> {
    decode::<JwtClaims>(
        &jwt,
        &DecodingKey::from_rsa_pem(include_bytes!("../../config/public.pem")).unwrap(),
        &Validation::new(Algorithm::RS256)
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_jwt() {
        let session_uuid = uuid::Uuid::new_v4().to_string();
        let jwt = create_jwt(&session_uuid).unwrap();

        println!("{}", jwt);
        assert!(jwt.len() > 0);
    }

    #[test]
    fn test_validate_jwt() {
        let session_uuid = uuid::Uuid::new_v4().to_string();
        let jwt = create_jwt(&session_uuid).unwrap();
        let result = validate_jwt(jwt);

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_jwt_with_bad_signature() {
        let session_uuid = uuid::Uuid::new_v4().to_string();
        let jwt = create_jwt(&session_uuid).unwrap();
        let result = validate_jwt(format!("{}x", jwt));

        println!("{:?}", result);
        assert!(result.is_err());
    }
}
