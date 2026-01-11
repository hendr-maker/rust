//! Authentication service - Handles user authentication and authorization.
//!
//! SOLID (SRP): Handles authentication concerns only.
//! DDD: Uses domain Password value object for hashing.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::client::UserServiceClient;
use common::{AppError, AppResult};
use domain::{Password, User, SECONDS_PER_HOUR, TOKEN_TYPE_BEARER};

/// JWT claims payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

/// Token response returned after successful authentication
#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    /// JWT access token
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration time in seconds
    pub expires_in: i64,
}

/// Authentication service trait for dependency injection.
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Register a new user
    async fn register(&self, email: String, password: String, name: String) -> AppResult<User>;

    /// Login and return JWT token
    async fn login(&self, email: String, password: String) -> AppResult<TokenResponse>;

    /// Verify JWT token and extract claims
    fn verify_token(&self, token: &str) -> AppResult<Claims>;

    /// Refresh an existing token
    async fn refresh_token(&self, claims: &Claims) -> AppResult<TokenResponse>;
}

/// Concrete implementation of AuthService using gRPC client to user-service.
pub struct Authenticator {
    user_client: Arc<dyn UserServiceClient>,
    jwt_secret: String,
    jwt_expiration_hours: i64,
}

impl Authenticator {
    /// Create new auth service instance
    pub fn new(
        user_client: Arc<dyn UserServiceClient>,
        jwt_secret: String,
        jwt_expiration_hours: i64,
    ) -> Self {
        Self {
            user_client,
            jwt_secret,
            jwt_expiration_hours,
        }
    }

    /// Get JWT secret as bytes
    fn jwt_secret_bytes(&self) -> &[u8] {
        self.jwt_secret.as_bytes()
    }

    /// Generate JWT token for a user
    fn generate_token(&self, user: &User) -> AppResult<TokenResponse> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(self.jwt_expiration_hours);

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
            &EncodingKey::from_secret(self.jwt_secret_bytes()),
        )?;

        Ok(TokenResponse {
            access_token: token,
            token_type: TOKEN_TYPE_BEARER.to_string(),
            expires_in: self.jwt_expiration_hours * SECONDS_PER_HOUR,
        })
    }
}

#[async_trait]
impl AuthService for Authenticator {
    async fn register(&self, email: String, password: String, name: String) -> AppResult<User> {
        // Check if user already exists (including soft-deleted to prevent email reuse)
        if self
            .user_client
            .find_by_email_with_deleted(&email)
            .await?
            .is_some()
        {
            return Err(AppError::conflict("User"));
        }

        // DDD: Use Password value object for hashing
        let password_hash = Password::new(&password)
            .map_err(|e| AppError::validation(e.to_string()))?
            .into_string();

        self.user_client.create(email, password_hash, name).await
    }

    async fn login(&self, email: String, password: String) -> AppResult<TokenResponse> {
        let user_result = self.user_client.find_by_email(&email).await?;

        // SECURITY: Perform password verification even if user doesn't exist
        // to prevent timing attacks that could enumerate valid emails.
        // We use a dummy hash that will always fail verification.
        let dummy_hash =
            "$argon2id$v=19$m=19456,t=2,p=1$dummysalt123456$dummyhash1234567890123456789012";

        let (password_hash, user_exists) = match &user_result {
            Some(user) => (user.password_hash.as_str(), true),
            None => (dummy_hash, false),
        };

        // DDD: Use Password value object for verification
        let stored_password = Password::from_hash(password_hash);
        let password_valid = stored_password.verify(&password);

        // Only succeed if both user exists AND password is valid
        if !user_exists || !password_valid {
            return Err(AppError::InvalidCredentials);
        }

        // Safe to unwrap since we verified user_exists is true
        self.generate_token(user_result.as_ref().unwrap())
    }

    fn verify_token(&self, token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    async fn refresh_token(&self, claims: &Claims) -> AppResult<TokenResponse> {
        // Verify user still exists and is active, and get fresh user data
        // This ensures the new token has the current role (in case it changed)
        let user = self
            .user_client
            .find_by_email(&claims.email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        // Generate token from fresh user data, not stale claims
        self.generate_token(&user)
    }
}
