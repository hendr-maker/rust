//! Authentication handlers.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::api::extractors::ValidatedJson;
use crate::api::AppState;
use crate::domain::UserResponse;
use crate::errors::AppResult;
use crate::services::TokenResponse;

/// User registration request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    /// User email address
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User password (minimum 8 characters)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[schema(example = "SecurePass123!", min_length = 8)]
    pub password: String,
    /// User display name
    #[validate(length(min = 1, message = "Name is required"))]
    #[schema(example = "John Doe")]
    pub name: String,
}

/// User login request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// User email address
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User password
    #[schema(example = "SecurePass123!")]
    pub password: String,
}

/// Create authentication routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Authentication",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "User already exists")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let user = state
        .auth_service
        .register(payload.email, payload.password, payload.name)
        .await?;

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

/// Login and get JWT token
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    let token = state
        .auth_service
        .login(payload.email, payload.password)
        .await?;

    Ok(Json(token))
}
