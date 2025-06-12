use serde::Deserialize;

use crate::{
    db::repositories::user_repository::UserRepository,
    modules::{auth::auth_service::AuthService, user::user_service::UserService},
};

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: i64,
}

#[derive(Clone)]
pub struct Services {
    pub user_service: UserService,
    pub auth_service: AuthService,
}

#[derive(Clone)]
pub struct Repositories {
    pub user_repository: UserRepository,
}
