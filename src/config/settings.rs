//! Application settings loaded from environment variables.

use std::env;

use super::constants::{
    DEFAULT_DATABASE_URL, DEFAULT_JWT_EXPIRATION_HOURS, DEFAULT_REDIS_URL, DEFAULT_SERVER_HOST,
    DEFAULT_SERVER_PORT, MIN_JWT_SECRET_LENGTH,
};

/// Application configuration
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub server_host: String,
    pub server_port: u16,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &"[REDACTED]")
            .field("redis_url", &"[REDACTED]")
            .field("jwt_secret", &"[REDACTED]")
            .field("jwt_expiration_hours", &self.jwt_expiration_hours)
            .field("server_host", &self.server_host)
            .field("server_port", &self.server_port)
            .finish()
    }
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Panics
    /// Panics if JWT_SECRET is not set or is too short (security requirement).
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            if cfg!(debug_assertions) {
                // Development mode: use default but warn
                tracing::warn!("JWT_SECRET not set, using insecure default for development");
                "dev-secret-key-minimum-32-chars!!".to_string()
            } else {
                // Production mode: panic
                panic!("JWT_SECRET environment variable must be set in production");
            }
        });

        // Validate JWT secret length
        if jwt_secret.len() < MIN_JWT_SECRET_LENGTH {
            panic!(
                "JWT_SECRET must be at least {} characters long",
                MIN_JWT_SECRET_LENGTH
            );
        }

        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string()),
            jwt_secret,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_JWT_EXPIRATION_HOURS),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| DEFAULT_SERVER_HOST.to_string()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(DEFAULT_SERVER_PORT),
        }
    }

    /// Get JWT secret bytes for token signing/verification.
    pub fn jwt_secret_bytes(&self) -> &[u8] {
        self.jwt_secret.as_bytes()
    }

    /// Get the full server address.
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}
