use crate::core::base::repository::{Repository};
use uuid::Uuid;
use crate::db::user::user_model::User;

pub type UserRepository = Repository<User>;

impl Repository<User> {

    pub async fn create_user(&self, email: &str, username: &str, hashed_password: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, username, password) VALUES ($1, $2, $3) 
                RETURNING id, email, username, password_hash, created_at, updated_at"
        )
            .bind(email)
            .bind(username)
            .bind(hashed_password)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    // Exemple de mise à jour partielle dynamique
    pub async fn update_username(&self, id: Uuid, new_username: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
            .bind(new_username) // $1
            .bind(id)           // $2
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}