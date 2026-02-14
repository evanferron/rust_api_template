use crate::config::models::Repositories;
use crate::core::utils::bcrypt::hash_password;
use crate::db::user::user_model::User;
use crate::{core::errors::errors::ApiError, modules::user::user_dto::CreateUserRequest};
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
        self.repositories
            .user_repository
            .find_all()
            .await
            .map_err(|e| ApiError::InternalServer(e.to_string()))
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<User, ApiError> {
        let user = self.repositories.user_repository.find_by_id(id).await?;
        if let Some(user) = user {
            Ok(user)
        } else {
            Err(ApiError::NotFound(format!(
                "Utilisateur avec ID {} non trouvé",
                id
            )))
        }
    }

    pub async fn create_user(&self, user: CreateUserRequest) -> Result<User, ApiError> {
        let existing_user = self
            .repositories
            .user_repository
            .find_by(vec![("email", user.email.clone().into())])
            .await?;

        if !existing_user.is_empty() {
            return Err(ApiError::Conflict(format!(
                "Un utilisateur avec l'email {} existe déjà",
                user.email
            )));
        }

        // Password hashing
        let password_hash = hash_password(&user.password)?;

        // Persist the user
        Ok(self
            .repositories
            .user_repository
            .create_user(&user.username, &user.email, &password_hash)
            .await?)
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
                let existing_users = self
                    .repositories
                    .user_repository
                    .find_by(vec![("email", new_email.clone().into())])
                    .await?;
                if !existing_users.is_empty() {
                    return Err(ApiError::Conflict(format!(
                        "Un utilisateur avec l'email {} existe déjà",
                        new_email
                    )));
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
            .update(
                id,
                vec![
                    ("username", user.username.clone().into()),
                    ("email", user.email.clone().into()),
                    ("password_hash", user.password_hash.clone().into()),
                ],
            )
            .await
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<bool, ApiError> {
        // Check if the user exists
        self.get_user_by_id(id).await?;

        // Delete the user
        self.repositories.user_repository.delete(id).await
    }
}
