//! Authentication service - Handles user authentication and authorization.
//!
//! SOLID (SRP): Handles authentication concerns only.
//! SOLID (ISP): Trait contains only auth methods, password handling in domain.
//! DDD: Uses domain Password value object for hashing.
//! DDD: Uses Unit of Work for repository access.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::config::{Config, SECONDS_PER_HOUR, TOKEN_TYPE_BEARER};
use crate::domain::{Password, User};
use crate::errors::{AppError, AppResult};
use crate::infra::UnitOfWork;

/// JWT claims payload
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

/// Token response returned after successful authentication
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    /// JWT access token
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub access_token: String,
    /// Token type (always "Bearer")
    #[schema(example = "Bearer")]
    pub token_type: String,
    /// Token expiration time in seconds
    #[schema(example = 86400)]
    pub expires_in: i64,
}

/// Authentication service trait for dependency injection.
///
/// SOLID (ISP): Contains only authentication operations.
/// Password hashing is handled by domain::Password value object.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Register a new user
    async fn register(&self, email: String, password: String, name: String) -> AppResult<User>;

    /// Login and return JWT token
    async fn login(&self, email: String, password: String) -> AppResult<TokenResponse>;

    /// Verify JWT token and extract claims
    fn verify_token(&self, token: &str) -> AppResult<Claims>;
}

/// Generate JWT token for a user (shared helper to avoid duplication)
fn generate_token(user: &User, config: &Config) -> AppResult<TokenResponse> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(config.jwt_expiration_hours);

    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        role: user.role.to_string(),
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret_bytes()),
    )?;

    Ok(TokenResponse {
        access_token: token,
        token_type: TOKEN_TYPE_BEARER.to_string(),
        expires_in: config.jwt_expiration_hours * SECONDS_PER_HOUR,
    })
}

/// Verify JWT token and extract claims (shared helper)
fn verify_token_internal(token: &str, config: &Config) -> AppResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Concrete implementation of AuthService using Unit of Work.
pub struct Authenticator<U: UnitOfWork> {
    uow: Arc<U>,
    config: Config,
}

impl<U: UnitOfWork> Authenticator<U> {
    /// Create new auth service instance with Unit of Work
    pub fn new(uow: Arc<U>, config: Config) -> Self {
        Self { uow, config }
    }
}

#[async_trait]
impl<U: UnitOfWork> AuthService for Authenticator<U> {
    async fn register(&self, email: String, password: String, name: String) -> AppResult<User> {
        // Email format is validated by the handler's ValidatedJson extractor
        // Check if user already exists (including soft-deleted to prevent email reuse)
        if self.uow.users().find_by_email_with_deleted(&email).await?.is_some() {
            return Err(AppError::conflict("User"));
        }

        // DDD: Use Password value object for hashing
        let password_hash = Password::new(&password)?.into_string();
        self.uow.users().create(email, password_hash, name).await
    }

    async fn login(&self, email: String, password: String) -> AppResult<TokenResponse> {
        let user_result = self.uow.users().find_by_email(&email).await?;

        // SECURITY: Perform password verification even if user doesn't exist
        // to prevent timing attacks that could enumerate valid emails.
        // We use a dummy hash that will always fail verification.
        let dummy_hash = "$argon2id$v=19$m=19456,t=2,p=1$dummysalt123456$dummyhash1234567890123456789012";

        let (password_hash, user_exists) = match &user_result {
            Some(user) => (user.password_hash.as_str(), true),
            None => (dummy_hash, false),
        };

        // DDD: Use Password value object for verification
        let stored_password = Password::from_hash(password_hash.to_string());
        let password_valid = stored_password.verify(&password);

        // Only succeed if both user exists AND password is valid
        if !user_exists || !password_valid {
            return Err(AppError::InvalidCredentials);
        }

        // Safe to unwrap since we verified user_exists is true
        generate_token(user_result.as_ref().unwrap(), &self.config)
    }

    fn verify_token(&self, token: &str) -> AppResult<Claims> {
        verify_token_internal(token, &self.config)
    }
}
