//! Auth service configuration.

use std::env;

/// Auth service configuration.
#[derive(Debug, Clone)]
pub struct AuthServiceConfig {
    /// JWT secret for signing tokens (min 32 characters)
    pub jwt_secret: String,
    /// JWT token expiration in hours
    pub jwt_expiration_hours: i64,
    /// User service gRPC endpoint
    pub user_service_url: String,
    /// Redis URL for session management
    pub redis_url: String,
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
}

impl AuthServiceConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET")
                .or_else(|_| env::var("AUTH_SERVICE_JWT_SECRET"))
                .expect("JWT_SECRET must be set (minimum 32 characters)"),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .or_else(|_| env::var("AUTH_SERVICE_JWT_EXPIRATION_HOURS"))
                .ok()
                .and_then(|h| h.parse().ok())
                .unwrap_or(24),
            user_service_url: env::var("USER_SERVICE_URL")
                .or_else(|_| env::var("AUTH_SERVICE_USER_SERVICE_URL"))
                .unwrap_or_else(|_| "http://localhost:50052".to_string()),
            redis_url: env::var("AUTH_SERVICE_REDIS_URL")
                .or_else(|_| env::var("REDIS_URL"))
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            host: env::var("AUTH_SERVICE_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("AUTH_SERVICE_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(50051),
        }
    }

    /// Get JWT secret as bytes.
    pub fn jwt_secret_bytes(&self) -> &[u8] {
        self.jwt_secret.as_bytes()
    }
}

impl Default for AuthServiceConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            jwt_expiration_hours: 24,
            user_service_url: "http://localhost:50052".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            host: "0.0.0.0".to_string(),
            port: 50051,
        }
    }
}
