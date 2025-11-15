use crate::core::base::generic_repository::repository_trait::{RepositoryTrait};
use crate::core::errors::errors::ApiError;
use crate::db::models::user::User;
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;
use crate::core::base::generic_repository::entry_trait::Entry;
use crate::core::base::query_builder::query_builder::{DbType, QueryBuilder};

#[derive(Clone)]
pub struct UserRepository {
    pool: Pool<Postgres>,
}

impl UserRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApiError> {
        let user = self.find_by_column("email", email.into()).await?;
        Ok(user.into_iter().next())
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, ApiError> {
        let user = self.find_by_column("username", username.into()).await?;
        Ok(user.into_iter().next())
    }

    pub async fn update_password(
        &self,
        id: Uuid,
        new_password_hash: &str,
    ) -> Result<User, ApiError> {
        self.update(
            id.into(),
            vec!["password"],
            vec![new_password_hash.into()],
        ).await
    }
}

// Implementation of the RepositoryTrait for UserRepository
impl RepositoryTrait<User, Postgres> for UserRepository {
    fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    fn query(&self) -> QueryBuilder<Postgres, User> {
        QueryBuilder::new(DbType::Postgres)
    }

    fn query_custom<M>(&self) -> QueryBuilder<Postgres, M>
    where
        M: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
    {
        QueryBuilder::new(DbType::Postgres)
    }

    // You can override trait methods if needed
    // For example, to customize find_all with a specific ordering:
    async fn find_all(&self) -> Result<Vec<User>, ApiError> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }
}

// Facade implementation for UserRepository
impl UserRepository {
    pub async fn find_all_users(&self) -> Result<Vec<User>,ApiError> {
        self.find_all().await
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Result<User,ApiError> {
        self.find_by_id(id.into()).await
    }

    pub async fn create_user(&self, user: User) -> Result<User,ApiError> {
        self.create(user).await
    }

    pub async fn update_user(&self, id: Uuid, user: User) -> Result<User,ApiError> {
        self.update(id.into(), vec!["email","username" ], vec![user.email.into(),user.username.into()]).await
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<bool, ApiError> {
        self.delete(id.into()).await
    }
}
