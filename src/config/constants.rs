//! Application-wide constants
//!
//! Centralized location for magic values to improve maintainability.

// =============================================================================
// Pagination
// =============================================================================

/// Default number of items per page
pub const DEFAULT_PAGE_SIZE: u64 = 20;

/// Maximum allowed items per page to prevent excessive queries
pub const MAX_PAGE_SIZE: u64 = 100;

/// Default starting page number (1-indexed)
pub const DEFAULT_PAGE_NUMBER: u64 = 1;

// =============================================================================
// Authentication & Security
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
// Server Configuration
// =============================================================================

/// Default server host address
pub const DEFAULT_SERVER_HOST: &str = "0.0.0.0";

/// Default server port
pub const DEFAULT_SERVER_PORT: u16 = 3000;

// =============================================================================
// Database
// =============================================================================

/// Default database connection URL (for development)
pub const DEFAULT_DATABASE_URL: &str = "postgres://postgres:password@localhost:5432/rust_app";

// =============================================================================
// Cache (Redis)
// =============================================================================

/// Default Redis URL (for development)
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

/// Default cache TTL in seconds (1 hour)
pub const DEFAULT_CACHE_TTL_SECONDS: u64 = 3600;

/// Cache key prefix for user data
pub const CACHE_PREFIX_USER: &str = "user:";

/// Cache key prefix for session data
pub const CACHE_PREFIX_SESSION: &str = "session:";

/// Cache key prefix for rate limiting
pub const CACHE_PREFIX_RATE_LIMIT: &str = "rate_limit:";

// =============================================================================
// Distributed Locks & Semaphores
// =============================================================================

/// Cache key prefix for distributed locks
pub const CACHE_PREFIX_LOCK: &str = "lock:";

/// Cache key prefix for semaphores
pub const CACHE_PREFIX_SEMAPHORE: &str = "semaphore:";

/// Default lock TTL in seconds (prevents deadlocks)
pub const DEFAULT_LOCK_TTL_SECONDS: u64 = 30;

/// Default lock retry attempts
pub const DEFAULT_LOCK_RETRIES: u32 = 10;

/// Default lock retry delay in milliseconds
pub const DEFAULT_LOCK_RETRY_DELAY_MS: u64 = 100;

// =============================================================================
// Rate Limiting
// =============================================================================

/// Default rate limit: requests per window
pub const RATE_LIMIT_REQUESTS: u64 = 100;

/// Default rate limit window in seconds (1 minute)
pub const RATE_LIMIT_WINDOW_SECONDS: u64 = 60;

/// Stricter rate limit for auth endpoints: requests per window
pub const RATE_LIMIT_AUTH_REQUESTS: u64 = 10;

/// Auth rate limit window in seconds (1 minute)
pub const RATE_LIMIT_AUTH_WINDOW_SECONDS: u64 = 60;

// =============================================================================
// Background Jobs
// =============================================================================

/// Email job queue identifier
pub const JOB_NAME_EMAIL: &str = "email::send";

// =============================================================================
// Validation
// =============================================================================

/// Minimum password length requirement
pub const MIN_PASSWORD_LENGTH: u64 = 8;

/// Minimum name length requirement
pub const MIN_NAME_LENGTH: u64 = 1;
