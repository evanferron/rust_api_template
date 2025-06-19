use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// jwt models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user: Sub,
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}
#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct Sub {
    pub id: Uuid,
    pub email: String,
    pub is_admin: Option<bool>,
}

// end jwt models

// request models
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 100))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 100))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RefreshRequest {
    #[validate(length(min = 1))]
    pub refresh_token: String,
}

// end request models

// response models
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshResponse {
    pub token: String,
    pub refresh_token: String,
}
// end response models
