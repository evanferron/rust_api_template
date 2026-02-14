use std::sync::Arc;

use tokio::task;

use crate::db::user::user_model::User;
use crate::{
    config::models::Repositories,
    core::errors::errors::ApiError,
    modules::auth::{auth_helpers::verify_password, auth_models::RegisterRequest},
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

    pub async fn create_user(&self, user: RegisterRequest) -> Result<User, ApiError> {
        if !self
            .repositories
            .user_repository
            .find_by(vec![("email", user.email.clone().into())])
            .await?
            .is_empty()
        {
            return Err(ApiError::BadRequest(
                "Un utilisateur avec cet email existe déjà".to_string(),
            ));
        }

        let password_hash = task::spawn_blocking(move || bcrypt::hash(&user.password, 8))
            .await
            .map_err(|e| ApiError::InternalServer(format!("Erreur de tâche: {}", e)))?
            .map_err(|e| {
                ApiError::InternalServer(format!("Erreur de hash du mot de passe: {}", e))
            })?;

        let created_user = self
            .repositories
            .user_repository
            .create_user(&user.username, &user.email, &password_hash)
            .await?;

        Ok(created_user)
    }

    pub async fn authenticate_user(
        &self,
        email: String,
        password: String,
    ) -> Result<User, ApiError> {
        let users = self
            .repositories
            .user_repository
            .find_by(vec![("email", email.clone().into())])
            .await?;
        if users.is_empty() {
            return Err(ApiError::Authentication(
                "Email ou mot de passe invalide".to_string(),
            ));
        };
        let user = users.into_iter().next().unwrap();

        let password_hash = user.password_hash.clone();
        let password_verification =
            task::spawn_blocking(move || verify_password(&password, &password_hash))
                .await
                .map_err(|e| ApiError::InternalServer(format!("Erreur de tâche: {}", e)))??;

        if !password_verification {
            return Err(ApiError::Authentication(
                "Email ou mot de passe invalide".to_string(),
            ));
        }

        Ok(user)
    }
}
