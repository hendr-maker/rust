//! User service configuration.

use std::env;

/// User service configuration.
#[derive(Debug, Clone)]
pub struct UserServiceConfig {
    /// Database connection URL
    pub database_url: String,
    /// Redis URL for caching
    pub redis_url: String,
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
}

impl UserServiceConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("USER_SERVICE_DATABASE_URL")
                .or_else(|_| env::var("DATABASE_URL"))
                .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/user_db".to_string()),
            redis_url: env::var("USER_SERVICE_REDIS_URL")
                .or_else(|_| env::var("REDIS_URL"))
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            host: env::var("USER_SERVICE_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("USER_SERVICE_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(50052),
        }
    }
}

impl Default for UserServiceConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://postgres:password@localhost:5432/user_db".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            host: "0.0.0.0".to_string(),
            port: 50052,
        }
    }
}
