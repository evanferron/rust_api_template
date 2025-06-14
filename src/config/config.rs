use std::env;

use serde::Deserialize;

use crate::config::models::{DatabaseConfig, JwtConfig, ServerConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let server = ServerConfig {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse::<u16>()
                .unwrap_or(5432),
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        };

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL").expect("DATABASE_URL doit être définie"),
        };

        let jwt = JwtConfig {
            secret: env::var("JWT_SECRET").expect("JWT_SECRET doit être définie"),
            expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string())
                .parse::<i64>()
                .unwrap_or(86400),
            refresh_secret: env::var("JWT_REFRESH_SECRET")
                .expect("JWT_REFRESH_SECRET doit être définie"),
            refresh_expiration: env::var("JWT_REFRESH_EXPIRATION")
                .unwrap_or_else(|_| "604800".to_string())
                .parse::<i64>()
                .unwrap_or(604800),
        };

        Ok(Config {
            server,
            database,
            jwt,
        })
    }
}
