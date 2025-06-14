use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub token: String,
    pub refresh_token: String,
}
