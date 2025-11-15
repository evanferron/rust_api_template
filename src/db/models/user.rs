use crate::core::base::generic_repository::entry_trait::{BindValue, Entry};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }
}

impl<DB> Entry<DB> for User
where
    Uuid: sqlx::Encode<'static, DB>,
    Uuid: sqlx::Type<DB>,
    DB: sqlx::Database
{
    type Id = Uuid;

    fn set_created_at(&mut self, created_at: DateTime<Utc>) {
        self.created_at = created_at;
    }

    fn set_updated_at(&mut self, updated_at: DateTime<Utc>) {
        self.updated_at = updated_at;
    }

    fn table_name() -> &'static str {
        "users"
    }

    fn columns() -> Vec<&'static str> {
        vec![
            "id",
            "username",
            "email",
            "password_hash",
            "created_at",
            "updated_at",
        ]
    }

    fn insertable_columns() -> Vec<&'static str> {
        vec![
            "username",
            "email",
            "password_hash",
            "updated_at",
        ]
    }

    fn to_bind_values(&self) -> Vec<BindValue> {
        vec![
            self.id.into(),
            self.username.clone().into(),
            self.email.clone().into(),
            self.password_hash.clone().into(),
            self.created_at.into(),
            self.updated_at.into(),
        ]
    }
}
