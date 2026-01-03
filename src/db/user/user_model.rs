use chrono::{DateTime, Utc};
use crate::core::base::repository::DbEntity;
use serde::{Serialize};
use sqlx::{FromRow};
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DbEntity for User {
    fn table_name() -> &'static str { "users" }
}