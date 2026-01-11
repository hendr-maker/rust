//! Domain-level errors.
//!
//! These errors represent business rule violations and domain logic failures.
//! They are independent of infrastructure concerns (HTTP, gRPC, database).

use thiserror::Error;

/// Domain-specific errors for business rule violations.
#[derive(Error, Debug, Clone)]
pub enum DomainError {
    /// Validation failed for a field or input
    #[error("Validation error: {0}")]
    Validation(String),

    /// Password-related errors
    #[error("Password error: {0}")]
    Password(String),

    /// Entity not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Entity already exists (conflict)
    #[error("{0} already exists")]
    Conflict(String),

    /// Unauthorized access attempt
    #[error("Unauthorized")]
    Unauthorized,

    /// Forbidden action
    #[error("Forbidden")]
    Forbidden,

    /// Invalid credentials provided
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Internal domain error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl DomainError {
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        DomainError::Validation(msg.into())
    }

    /// Create a password error
    pub fn password(msg: impl Into<String>) -> Self {
        DomainError::Password(msg.into())
    }

    /// Create a not found error
    pub fn not_found(entity: impl Into<String>) -> Self {
        DomainError::NotFound(entity.into())
    }

    /// Create a conflict error
    pub fn conflict(entity: impl Into<String>) -> Self {
        DomainError::Conflict(entity.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        DomainError::Internal(msg.into())
    }
}

/// Result type alias for domain operations
pub type DomainResult<T> = Result<T, DomainError>;
