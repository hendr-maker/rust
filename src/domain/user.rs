//! User domain entity and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::config::{ROLE_ADMIN, ROLE_USER};

/// User roles enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

impl UserRole {
    /// Check if this role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Check if this role can access a required role
    pub fn can_access(&self, required: &UserRole) -> bool {
        match self {
            UserRole::Admin => true,
            UserRole::User => matches!(required, UserRole::User),
        }
    }
}

impl From<&str> for UserRole {
    fn from(s: &str) -> Self {
        match s {
            ROLE_ADMIN => UserRole::Admin,
            _ => UserRole::User,
        }
    }
}

impl From<UserRole> for String {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::Admin => ROLE_ADMIN.to_string(),
            UserRole::User => ROLE_USER.to_string(),
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "{}", ROLE_ADMIN),
            UserRole::User => write!(f, "{}", ROLE_USER),
        }
    }
}

/// User domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp (None = active, Some = deleted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    /// Create a new user with default role
    pub fn new(id: Uuid, email: String, password_hash: String, name: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            email,
            password_hash,
            name,
            role: UserRole::User,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Check if user has admin role
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    /// Check if user is soft deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if user is active (not deleted)
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }

    /// Update user's name
    pub fn update_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Update user's role
    pub fn update_role(&mut self, role: UserRole) {
        self.role = role;
        self.updated_at = Utc::now();
    }

    /// Soft delete the user
    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Restore a soft-deleted user
    pub fn restore(&mut self) {
        self.deleted_at = None;
        self.updated_at = Utc::now();
    }
}

/// User creation data transfer object
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateUser {
    /// User email address
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User password (minimum 8 characters)
    #[schema(example = "SecurePass123!", min_length = 8)]
    pub password: String,
    /// User display name
    #[schema(example = "John Doe")]
    pub name: String,
}

/// User update data transfer object
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateUser {
    /// New display name
    #[schema(example = "Jane Doe")]
    pub name: Option<String>,
    /// New role (admin only)
    #[schema(example = "admin")]
    pub role: Option<String>,
}

/// User response (safe to return to client)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserResponse {
    /// Unique user identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// User email address
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User display name
    #[schema(example = "John Doe")]
    pub name: String,
    /// User role
    #[schema(example = "user")]
    pub role: String,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role.to_string(),
            created_at: user.created_at,
        }
    }
}
