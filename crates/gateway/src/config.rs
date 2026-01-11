//! Gateway configuration.

use std::env;

/// Gateway configuration.
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Auth service gRPC endpoint
    pub auth_service_url: String,
    /// User service gRPC endpoint
    pub user_service_url: String,
    /// Redis URL for caching and rate limiting
    pub redis_url: String,
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Rate limit: requests per window
    pub rate_limit_requests: u64,
    /// Rate limit window in seconds
    pub rate_limit_window_seconds: u64,
    /// Auth rate limit: requests per window
    pub rate_limit_auth_requests: u64,
    /// Auth rate limit window in seconds
    pub rate_limit_auth_window_seconds: u64,
}

impl GatewayConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:50051".to_string()),
            user_service_url: env::var("USER_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:50052".to_string()),
            redis_url: env::var("GATEWAY_REDIS_URL")
                .or_else(|_| env::var("REDIS_URL"))
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            host: env::var("GATEWAY_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("GATEWAY_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(100),
            rate_limit_window_seconds: env::var("RATE_LIMIT_WINDOW_SECONDS")
                .ok()
                .and_then(|w| w.parse().ok())
                .unwrap_or(60),
            rate_limit_auth_requests: env::var("RATE_LIMIT_AUTH_REQUESTS")
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(10),
            rate_limit_auth_window_seconds: env::var("RATE_LIMIT_AUTH_WINDOW_SECONDS")
                .ok()
                .and_then(|w| w.parse().ok())
                .unwrap_or(60),
        }
    }

    /// Extract auth service port from URL.
    pub fn auth_port(&self) -> u16 {
        self.auth_service_url
            .rsplit(':')
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(50051)
    }

    /// Extract user service port from URL.
    pub fn user_port(&self) -> u16 {
        self.user_service_url
            .rsplit(':')
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(50052)
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            auth_service_url: "http://localhost:50051".to_string(),
            user_service_url: "http://localhost:50052".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            host: "0.0.0.0".to_string(),
            port: 3000,
            rate_limit_requests: 100,
            rate_limit_window_seconds: 60,
            rate_limit_auth_requests: 10,
            rate_limit_auth_window_seconds: 60,
        }
    }
}
