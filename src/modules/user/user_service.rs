use crate::config::models::Repositories;
use crate::db::models::user::User;
use crate::{core::errors::errors::ApiError, modules::user::user_models::CreateUserRequest};
use bcrypt::{DEFAULT_COST, hash};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    pub repositories: Arc<Repositories>,
}

impl UserService {
    pub fn new(repositories: Arc<Repositories>) -> Self {
        UserService { repositories }
    }

    pub async fn get_users(&self) -> Result<Vec<User>, ApiError> {
        self.repositories.user_repository.find_all_users().await
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<User, ApiError> {
        let user = self
            .repositories
            .user_repository
            .find_user_by_id(id)
            .await?;

        match user {
            Some(user) => Ok(user),
            None => Err(ApiError::NotFound(format!(
                "Utilisateur avec l'ID {} non trouvé",
                id
            ))),
        }
    }

    pub async fn create_user(&self, user: CreateUserRequest) -> Result<User, ApiError> {
        // Check if the email already exists
        if let Some(_) = self
            .repositories
            .user_repository
            .find_by_email(&user.email)
            .await?
        {
            return Err(ApiError::Conflict(format!(
                "Un utilisateur avec l'email {} existe déjà",
                user.email
            )));
        }

        // Password hashing
        let password_hash = hash_password(&user.password)?;

        // Create the user
        let user = User::new(user.username, user.email, password_hash);

        // Persist the user
        self.repositories.user_repository.create_user(user).await
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        username: Option<String>,
        email: Option<String>,
        password: Option<String>,
    ) -> Result<User, ApiError> {
        // Retrieve the existing user
        let mut user = self.get_user_by_id(id).await?;

        // Update fields if provided
        if let Some(new_username) = username {
            user.username = new_username;
        }

        if let Some(new_email) = email {
            // Check if the new email is already used by another user
            if new_email != user.email {
                if let Some(existing) = self
                    .repositories
                    .user_repository
                    .find_by_email(&new_email)
                    .await?
                {
                    if existing.id != id {
                        return Err(ApiError::Conflict(format!(
                            "Un utilisateur avec l'email {} existe déjà",
                            new_email
                        )));
                    }
                }
                user.email = new_email;
            }
        }

        if let Some(new_password) = password {
            user.password_hash = hash_password(&new_password)?;
        }

        // Update the user
        self.repositories
            .user_repository
            .update_user(id, user)
            .await
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<bool, ApiError> {
        // Check if the user exists
        self.get_user_by_id(id).await?;

        // Delete the user
        self.repositories.user_repository.delete_user(id).await
    }
}

// Utility functions for password handling
fn hash_password(password: &str) -> Result<String, ApiError> {
    hash(password, DEFAULT_COST)
        .map_err(|e| ApiError::InternalServer(format!("Échec du hashage du mot de passe: {}", e)))
}
