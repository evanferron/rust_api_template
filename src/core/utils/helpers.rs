use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // ID de l'utilisateur
    pub exp: i64,    // Expiration timestamp
    pub iat: i64,    // Issued at timestamp
}

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

pub fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

pub fn slugify(text: &str) -> String {
    let mut slug = text.to_lowercase();
    // Remplace les caractères spéciaux par des tirets
    slug = slug
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() {
                '-'
            } else {
                '-'
            }
        })
        .collect();

    // Supprime les tirets consécutifs
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }

    // Supprime les tirets au début ou à la fin
    if slug.starts_with('-') {
        slug.remove(0);
    }
    if slug.ends_with('-') {
        slug.pop();
    }

    slug
}
