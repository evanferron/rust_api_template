use std::sync::Arc;

use bcrypt::verify;

use crate::{config::models::Repositories, core::errors::errors::ApiError, db::models::user::User};

#[derive(Clone)]
pub struct AuthService {
    pub repositories: Arc<Repositories>,
}

impl AuthService {
    pub fn new(repository: Arc<Repositories>) -> Self {
        AuthService {
            repositories: repository,
        }
    }

    pub async fn authenticate_user(
        &self,
        email: String,
        password: String,
    ) -> Result<User, ApiError> {
        // Récupération de l'utilisateur par email
        let user = match self
            .repositories
            .user_repository
            .find_by_email(&email)
            .await?
        {
            Some(user) => user,
            None => {
                return Err(ApiError::Authentication(
                    "Email ou mot de passe invalide".to_string(),
                ));
            }
        };

        // Vérification du mot de passe
        if !verify_password(&password, &user.password_hash)? {
            return Err(ApiError::Authentication(
                "Email ou mot de passe invalide".to_string(),
            ));
        }

        // Vérification si le compte est actif
        if !user.is_active {
            return Err(ApiError::Authorization(
                "Votre compte est désactivé".to_string(),
            ));
        }

        Ok(user)
    }
}
fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    verify(password, hash).map_err(|e| {
        ApiError::InternalServer(format!("Échec de la vérification du mot de passe: {}", e))
    })
}
