use std::sync::Arc;

use bcrypt::verify;

use crate::{
    config::models::Repositories, core::errors::errors::ApiError, db::models::user::User,
    modules::user::user_models::CreateUserRequest,
};

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

    pub async fn create_user(&self, user: CreateUserRequest) -> Result<User, ApiError> {
        // Vérification si l'utilisateur existe déjà
        if self
            .repositories
            .user_repository
            .find_by_email(&user.email)
            .await?
            .is_some()
        {
            return Err(ApiError::Validation(
                "Un utilisateur avec cet email existe déjà".to_string(),
            ));
        }
        // Création de l'utilisateur
        let user = User::new(user.username, user.email, user.password);
        let created_user = self.repositories.user_repository.create_user(user).await?;

        Ok(created_user)
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

        Ok(user)
    }
}
fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    verify(password, hash).map_err(|e| {
        ApiError::InternalServer(format!("Échec de la vérification du mot de passe: {}", e))
    })
}
