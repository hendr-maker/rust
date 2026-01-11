//! Centralized error handling.
//!
//! Provides a unified error type for the entire application,
//! with automatic HTTP response conversion.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Application error types
/// SOLID - Open/Closed: Extend via new variants without modifying behavior
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

    // External service errors
    #[error("Database error")]
    Database(#[from] sea_orm::DbErr),

    #[error("Authentication error")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    // Internal
    #[error("Internal server error")]
    Internal(String),
}

/// Error response body
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
    fn code(&self) -> &'static str {
        match self {
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden => "FORBIDDEN",
            AppError::InvalidCredentials => "INVALID_CREDENTIALS",
            AppError::NotFound => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Jwt(_) => "AUTH_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Get HTTP status code
    fn status(&self) -> StatusCode {
        match self {
            AppError::Unauthorized | AppError::InvalidCredentials | AppError::Jwt(_) => {
                StatusCode::UNAUTHORIZED
            }
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Database(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get user-facing message (hides internal details)
    fn user_message(&self) -> String {
        match self {
            // Show full message for client errors
            AppError::Validation(msg) => msg.clone(),
            AppError::BadRequest(msg) => msg.clone(),
            AppError::Conflict(msg) => format!("{} already exists", msg),

            // Hide details for internal/security errors
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                "A database error occurred".to_string()
            }
            AppError::Jwt(e) => {
                tracing::error!("JWT error: {:?}", e);
                "Invalid or expired token".to_string()
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                "An internal error occurred".to_string()
            }

            // Use default message for others
            _ => self.to_string(),
        }
    }
}

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
}
