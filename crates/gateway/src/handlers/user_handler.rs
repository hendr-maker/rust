//! User handlers.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use common::{AppError, AppResult};
use domain::{is_valid_role, UserResponse};

use crate::extractors::ValidatedJson;
use crate::middleware::{require_admin, CurrentUser};
use crate::state::AppState;

/// User update request with validation
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    /// New display name
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    #[schema(example = "Jane Doe")]
    pub name: Option<String>,
    /// New role (admin only)
    #[schema(example = "admin")]
    pub role: Option<String>,
}

/// Create user routes
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/me", get(get_current_user))
        .route("/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/:id/restore", post(restore_user))
}

/// Get current authenticated user
#[utoipa::path(
    get,
    path = "/users/me",
    tag = "Users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user profile", body = UserResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_current_user(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
) -> AppResult<Json<UserResponse>> {
    // Try cache first
    if let Some(user) = state.cache.get_user(&current_user.id).await? {
        return Ok(Json(UserResponse::from(user)));
    }

    // Cache miss - fetch from service
    let user = state.user_client.get_user(current_user.id).await?;

    // Cache for future requests
    state.cache.set_user(&user).await?;

    Ok(Json(UserResponse::from(user)))
}

/// List all users (admin only)
#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of all users", body = Vec<UserResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only")
    )
)]
pub async fn list_users(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<UserResponse>>> {
    require_admin(&current_user)?;
    let users = state.user_client.list_users().await?;
    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

/// Get user by ID (own profile or admin)
#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User profile", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Can only view own profile unless admin"),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    // Users can only view their own profile unless admin
    if current_user.id != id {
        require_admin(&current_user)?;
    }

    // Try cache first
    if let Some(user) = state.cache.get_user(&id).await? {
        return Ok(Json(UserResponse::from(user)));
    }

    // Cache miss - fetch from service
    let user = state.user_client.get_user(id).await?;

    // Cache for future requests
    state.cache.set_user(&user).await?;

    Ok(Json(UserResponse::from(user)))
}

/// Update user (own profile or admin for role changes)
#[utoipa::path(
    put,
    path = "/users/{id}",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Can only update own profile unless admin"),
        (status = 404, description = "User not found")
    )
)]
pub async fn update_user(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    // Users can only update their own profile
    if current_user.id != id {
        require_admin(&current_user)?;
    }

    // Only admin can change roles
    if payload.role.is_some() {
        require_admin(&current_user)?;
    }

    // Validate role value if provided
    if let Some(ref role) = payload.role {
        if !is_valid_role(role) {
            return Err(AppError::validation("Invalid role. Must be 'user' or 'admin'"));
        }
    }

    let user = state
        .user_client
        .update_user(id, payload.name, payload.role)
        .await?;

    // Update cache with new user data
    state.cache.set_user(&user).await?;

    Ok(Json(UserResponse::from(user)))
}

/// Delete user (admin only, cannot delete self)
#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 400, description = "Cannot delete your own account"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "User not found")
    )
)]
pub async fn delete_user(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    require_admin(&current_user)?;

    // Prevent self-deletion
    if current_user.id == id {
        return Err(AppError::validation("Cannot delete your own account"));
    }

    state.user_client.delete_user(id).await?;

    // Invalidate cache
    state.cache.invalidate_user(&id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Restore soft-deleted user (admin only)
#[utoipa::path(
    post,
    path = "/users/{id}/restore",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "User ID to restore")
    ),
    responses(
        (status = 200, description = "User restored successfully", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "User not found or not deleted")
    )
)]
pub async fn restore_user(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    require_admin(&current_user)?;

    let user = state.user_client.restore_user(id).await?;

    // Update cache with restored user
    state.cache.set_user(&user).await?;

    Ok(Json(UserResponse::from(user)))
}
