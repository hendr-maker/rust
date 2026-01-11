//! JWT authentication middleware.

use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::api::AppState;
use crate::config::{BEARER_TOKEN_PREFIX, ROLE_ADMIN};
use crate::errors::AppError;

/// Authenticated user extracted from JWT token
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

impl CurrentUser {
    /// Check if user has admin role.
    pub fn is_admin(&self) -> bool {
        self.role == ROLE_ADMIN
    }
}

/// JWT authentication middleware.
///
/// Extracts and validates the JWT token from the Authorization header,
/// then injects the CurrentUser into the request extensions.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix(BEARER_TOKEN_PREFIX)
        .ok_or(AppError::Unauthorized)?;

    let claims = state.auth_service.verify_token(token)?;

    let current_user = CurrentUser {
        id: claims.sub,
        email: claims.email,
        role: claims.role,
    };

    request.extensions_mut().insert(current_user);

    Ok(next.run(request).await)
}

/// Require admin role, returns Forbidden error if not admin.
pub fn require_admin(user: &CurrentUser) -> Result<(), AppError> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// Check if user has required role.
/// Note: Currently unused but kept for future role-based access patterns.
#[allow(dead_code)]
pub fn require_role(user: &CurrentUser, required_role: &str) -> Result<(), AppError> {
    if user.role == required_role || user.is_admin() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}
