//! Authentication middleware.

use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use common::{AppError, AppResult};
use domain::UserRole;

use crate::state::AppState;

/// Current authenticated user extracted from JWT.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
}

impl CurrentUser {
    /// Check if user has admin role.
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }
}

/// Check if user has admin privileges.
pub fn require_admin(user: &CurrentUser) -> AppResult<()> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// Authentication middleware that validates JWT tokens.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Extract token from Authorization header
    let token = extract_token(&request)?;

    // Verify token via auth-service
    let claims = state
        .auth_client
        .verify_token(&token)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Create current user from claims
    let current_user = CurrentUser {
        id: claims.user_id,
        email: claims.email,
        role: UserRole::from(claims.role),
    };

    // Insert current user into request extensions
    request.extensions_mut().insert(current_user);

    Ok(next.run(request).await)
}

/// Extract bearer token from Authorization header.
fn extract_token(request: &Request<Body>) -> AppResult<String> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized);
    }

    Ok(auth_header[7..].to_string())
}
