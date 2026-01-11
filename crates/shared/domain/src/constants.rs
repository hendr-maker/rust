//! Domain-level constants.
//!
//! These constants define business rules and validation requirements.

// =============================================================================
// User Roles
// =============================================================================

/// Default role assigned to new users
pub const ROLE_USER: &str = "user";

/// Administrator role with elevated privileges
pub const ROLE_ADMIN: &str = "admin";

/// All valid role values
pub const VALID_ROLES: &[&str] = &[ROLE_USER, ROLE_ADMIN];

/// Check if a role value is valid
pub fn is_valid_role(role: &str) -> bool {
    VALID_ROLES.contains(&role)
}

// =============================================================================
// Validation
// =============================================================================

/// Minimum password length requirement
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// Minimum name length requirement
pub const MIN_NAME_LENGTH: usize = 1;

// =============================================================================
// Authentication
// =============================================================================

/// Default JWT token expiration in hours
pub const DEFAULT_JWT_EXPIRATION_HOURS: i64 = 24;

/// Minimum JWT secret length (security requirement)
pub const MIN_JWT_SECRET_LENGTH: usize = 32;

/// Seconds per hour (for token expiration calculation)
pub const SECONDS_PER_HOUR: i64 = 3600;

/// Authorization header prefix for Bearer tokens
pub const BEARER_TOKEN_PREFIX: &str = "Bearer ";

/// JWT token type identifier
pub const TOKEN_TYPE_BEARER: &str = "Bearer";
