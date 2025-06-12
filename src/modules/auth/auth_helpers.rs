use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use uuid::Uuid;

use crate::modules::auth::auth_models::Claims;

pub fn generate_jwt(
    user_id: Uuid,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiration = now + chrono::Duration::seconds(expiration_seconds);

    let claims = Claims {
        sub: user_id.to_string(),
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
