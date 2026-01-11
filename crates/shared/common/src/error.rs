//! Unified error handling for HTTP and gRPC.
//!
//! Provides a single error type that can be converted to:
//! - Axum HTTP responses (for API gateway)
//! - Tonic gRPC status codes (for microservices)

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use domain::DomainError;
use serde::Serialize;
use thiserror::Error;
use tonic::Status;

/// Application error types with support for both HTTP and gRPC.
#[derive(Error, Debug)]
pub enum AppError {
    // Authentication & Authorization
    #[error("Authentication required")]
    Unauthorized,

    #[error("Access denied")]
    Forbidden,

    #[error("Invalid credentials")]
    InvalidCredentials,

    // Resource errors
    #[error("Resource not found")]
    NotFound,

    #[error("{0} already exists")]
    Conflict(String),

    // Validation
    #[error("{0}")]
    Validation(String),

    #[error("Invalid input: {0}")]
    BadRequest(String),

    // Rate limiting
    #[error("Too many requests")]
    TooManyRequests,

    // External service errors
    #[cfg(feature = "database")]
    #[error("Database error")]
    Database(#[from] sea_orm::DbErr),

    #[cfg(feature = "jwt")]
    #[error("Authentication error")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[cfg(feature = "cache")]
    #[error("Cache error")]
    Cache(#[from] redis::RedisError),

    // gRPC specific
    #[error("Service unavailable")]
    ServiceUnavailable(String),

    #[error("gRPC error: {0}")]
    Grpc(String),

    // Internal
    #[error("Internal server error")]
    Internal(String),
}

/// Error response body for HTTP
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

impl AppError {
    /// Get error code for client
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden => "FORBIDDEN",
            AppError::InvalidCredentials => "INVALID_CREDENTIALS",
            AppError::NotFound => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::TooManyRequests => "TOO_MANY_REQUESTS",
            #[cfg(feature = "database")]
            AppError::Database(_) => "DATABASE_ERROR",
            #[cfg(feature = "jwt")]
            AppError::Jwt(_) => "AUTH_ERROR",
            #[cfg(feature = "cache")]
            AppError::Cache(_) => "CACHE_ERROR",
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            AppError::Grpc(_) => "GRPC_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Get HTTP status code
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::Unauthorized | AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            #[cfg(feature = "jwt")]
            AppError::Jwt(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get user-facing message (hides internal details)
    pub fn user_message(&self) -> String {
        match self {
            // Show full message for client errors
            AppError::Validation(msg) => msg.clone(),
            AppError::BadRequest(msg) => msg.clone(),
            AppError::Conflict(msg) => {
                // Avoid duplicating "already exists" when converted from gRPC
                if msg.ends_with("already exists") {
                    msg.clone()
                } else {
                    format!("{} already exists", msg)
                }
            }

            // Hide details for internal/security errors
            #[cfg(feature = "database")]
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                "A database error occurred".to_string()
            }
            #[cfg(feature = "jwt")]
            AppError::Jwt(e) => {
                tracing::error!("JWT error: {:?}", e);
                "Invalid or expired token".to_string()
            }
            #[cfg(feature = "cache")]
            AppError::Cache(e) => {
                tracing::error!("Cache error: {:?}", e);
                "A cache error occurred".to_string()
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                "An internal error occurred".to_string()
            }
            AppError::ServiceUnavailable(service) => {
                tracing::error!("Service unavailable: {}", service);
                format!("Service {} is unavailable", service)
            }
            AppError::Grpc(msg) => {
                tracing::error!("gRPC error: {}", msg);
                "A service communication error occurred".to_string()
            }

            // Use default message for others
            _ => self.to_string(),
        }
    }
}

// =============================================================================
// HTTP Response (Axum)
// =============================================================================

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorResponse {
            error: ErrorBody {
                code: self.code().to_string(),
                message: self.user_message(),
            },
        };

        (status, Json(body)).into_response()
    }
}

// =============================================================================
// gRPC Status (Tonic)
// =============================================================================

impl From<AppError> for Status {
    fn from(err: AppError) -> Self {
        let code = match &err {
            AppError::Unauthorized | AppError::InvalidCredentials => tonic::Code::Unauthenticated,
            #[cfg(feature = "jwt")]
            AppError::Jwt(_) => tonic::Code::Unauthenticated,
            AppError::Forbidden => tonic::Code::PermissionDenied,
            AppError::NotFound => tonic::Code::NotFound,
            AppError::Conflict(_) => tonic::Code::AlreadyExists,
            AppError::Validation(_) | AppError::BadRequest(_) => tonic::Code::InvalidArgument,
            AppError::TooManyRequests => tonic::Code::ResourceExhausted,
            AppError::ServiceUnavailable(_) => tonic::Code::Unavailable,
            _ => tonic::Code::Internal,
        };

        Status::new(code, err.user_message())
    }
}

impl From<Status> for AppError {
    fn from(status: Status) -> Self {
        match status.code() {
            tonic::Code::Unauthenticated => AppError::Unauthorized,
            tonic::Code::PermissionDenied => AppError::Forbidden,
            tonic::Code::NotFound => AppError::NotFound,
            tonic::Code::AlreadyExists => AppError::Conflict(status.message().to_string()),
            tonic::Code::InvalidArgument => AppError::Validation(status.message().to_string()),
            tonic::Code::ResourceExhausted => AppError::TooManyRequests,
            tonic::Code::Unavailable => AppError::ServiceUnavailable(status.message().to_string()),
            _ => AppError::Grpc(status.message().to_string()),
        }
    }
}

// =============================================================================
// Domain Error Conversion
// =============================================================================

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Validation(msg) => AppError::Validation(msg),
            DomainError::Password(msg) => AppError::Validation(msg),
            DomainError::NotFound(_) => AppError::NotFound,
            DomainError::Conflict(msg) => AppError::Conflict(msg),
            DomainError::Unauthorized => AppError::Unauthorized,
            DomainError::Forbidden => AppError::Forbidden,
            DomainError::InvalidCredentials => AppError::InvalidCredentials,
            DomainError::Internal(msg) => AppError::Internal(msg),
        }
    }
}

/// Result type alias
pub type AppResult<T> = Result<T, AppError>;

/// Extension trait for Option -> AppError conversion
pub trait OptionExt<T> {
    fn ok_or_not_found(self) -> AppResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found(self) -> AppResult<T> {
        self.ok_or(AppError::NotFound)
    }
}

/// Convenience constructors
impl AppError {
    pub fn conflict(entity: impl Into<String>) -> Self {
        AppError::Conflict(entity.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        AppError::Validation(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        AppError::Internal(msg.into())
    }

    pub fn grpc(msg: impl Into<String>) -> Self {
        AppError::Grpc(msg.into())
    }

    pub fn service_unavailable(service: impl Into<String>) -> Self {
        AppError::ServiceUnavailable(service.into())
    }
}
