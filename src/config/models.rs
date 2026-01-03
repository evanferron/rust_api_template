use serde::Deserialize;

use crate::modules::{auth::auth_service::AuthService, user::user_service::UserService};
use crate::db::user::user_repository::UserRepository;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub acquire_timeout: u64, // seconds
    pub idle_timeout: u64,    // seconds
    pub max_lifetime: u64,    // seconds
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: u32,
    pub refresh_secret: String,
    pub refresh_expiration: u32,
}

pub struct Services {
    pub user_service: UserService,
    pub auth_service: AuthService,
}

pub struct Repositories {
    pub user_repository: UserRepository,
}
