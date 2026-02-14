use crate::core::base::repository::Repository;
use crate::db::user::user_model::User;

pub type UserRepository = Repository<User>;

impl Repository<User> {
    pub async fn create_user(
        &self,
        email: &str,
        username: &str,
        hashed_password: &str,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, username, password) VALUES ($1, $2, $3) 
                RETURNING id, email, username, password_hash, created_at, updated_at",
        )
        .bind(email)
        .bind(username)
        .bind(hashed_password)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}
