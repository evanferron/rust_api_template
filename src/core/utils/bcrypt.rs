use bcrypt::{hash, DEFAULT_COST};
use crate::core::errors::errors::ApiError;

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    hash(password, DEFAULT_COST)
        .map_err(|e| ApiError::InternalServer(format!("Échec du hashage du mot de passe: {}", e)))
}
