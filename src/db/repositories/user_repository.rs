use crate::core::base::generic_repository::repository_trait::RepositoryTrait;
use crate::core::errors::errors::ApiError;
use crate::db::models::user::User;
use chrono::Utc;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pool: Pool<Postgres>,
}

impl UserRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApiError> {
        let user = self.find_by_column("email", email).await?;
        Ok(user.into_iter().next())
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApiError> {
        let user = self.find_by_column("username", username).await?;
        Ok(user.into_iter().next())
    }

    pub async fn find_active_users(&self) -> Result<Vec<User>, ApiError> {
        let users = self.find_by_column("is_active", "true").await?;
        Ok(users)
    }

    pub async fn deactivate_user(&self, id: Uuid) -> Result<User, ApiError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET is_active = false, updated_at = $1
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(Utc::now())
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn activate_user(&self, id: Uuid) -> Result<User, ApiError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET is_active = true, updated_at = $1
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(Utc::now())
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_password(
        &self,
        id: Uuid,
        new_password_hash: &str,
    ) -> Result<User, ApiError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(new_password_hash)
        .bind(Utc::now())
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}

// Implémentation du trait RepositoryTrait pour UserRepository
impl RepositoryTrait<User> for UserRepository {
    fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    // Vous pouvez override les méthodes du trait si nécessaire
    // Par exemple, pour personnaliser find_all avec un tri spécifique :
    async fn find_all(&self) -> Result<Vec<User>, ApiError> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }
}

// Implémentation de conversions pour faciliter l'usage avec ApiError
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        // Adaptez cette conversion selon votre implémentation d'ApiError
        // Par exemple :
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("User not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.constraint().is_some() {
                    ApiError::Conflict("User already exists".to_string())
                } else {
                    ApiError::InternalServer("Database error".to_string())
                }
            }
            _ => ApiError::InternalServer("Database error".to_string()),
        }
    }
}

// Implémentation passerelle pour UserRepository
impl UserRepository {
    pub async fn find_all_users(&self) -> Result<Vec<User>, ApiError> {
        self.find_all().await
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>, ApiError> {
        self.find_by_id(id).await
    }

    pub async fn create_user(&self, user: User) -> Result<User, ApiError> {
        self.create(user).await
    }

    pub async fn update_user(&self, id: Uuid, user: User) -> Result<User, ApiError> {
        self.update(id, user).await
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<bool, ApiError> {
        self.delete(id).await
    }
}
