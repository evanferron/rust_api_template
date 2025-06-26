use bcrypt::verify;
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

use crate::{core::errors::errors::ApiError, modules::auth::auth_models::Claims};

pub fn generate_jwt(
    user: crate::modules::auth::auth_models::Sub,
    secret: &str,
    expiration_seconds: u32,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiration = now + chrono::Duration::seconds(expiration_seconds.into());

    let claims = Claims {
        sub: user.id.to_string(),
        user,
        exp: expiration.timestamp(),
        iat: now.timestamp(),
    };

    let header = Header::new(Algorithm::HS256);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    verify(password, hash).map_err(|e| {
        ApiError::InternalServer(format!("Échec de la vérification du mot de passe: {}", e))
    })
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, ApiError> {
    let validation = jsonwebtoken::Validation::new(Algorithm::HS256);
    match jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => Ok(data.claims),
        Err(e) => {
            eprintln!("[verify_token] JWT decode error: {e:?}");
            Err(ApiError::Authorization(format!(
                "Invalid or expired token: {e:?}"
            )))
        }
    }
}
