//! Shared configuration structures.

use serde::{Deserialize, Serialize};

/// Base service configuration shared by all services.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    /// Service name for logging and tracing
    pub service_name: String,
    /// Host address to bind
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Log level
    pub log_level: String,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            service_name: "service".to_string(),
            host: "0.0.0.0".to_string(),
            port: 3000,
            log_level: "info".to_string(),
        }
    }
}

/// Redis cache configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    pub url: String,
    pub default_ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            default_ttl_seconds: 3600,
        }
    }
}

/// Database configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://postgres:password@localhost:5432/rust_app".to_string(),
            max_connections: 10,
            min_connections: 1,
        }
    }
}

/// JWT configuration for authentication.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    #[serde(skip_serializing)]
    pub secret: String,
    pub expiration_hours: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: String::new(),
            expiration_hours: 24,
        }
    }
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u64,
    /// Window size in seconds
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

/// gRPC client connection configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GrpcClientConfig {
    /// Service endpoint URL (e.g., "http://localhost:50051")
    pub endpoint: String,
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
}

impl Default for GrpcClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:50051".to_string(),
            connect_timeout_ms: 5000,
            request_timeout_ms: 30000,
        }
    }
}
