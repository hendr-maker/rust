//! Middleware for authentication, rate limiting, and caching.

mod auth;
mod cache;
mod rate_limit;

pub use auth::{auth_middleware, require_admin, CurrentUser};
pub use cache::Cache;
pub use rate_limit::{rate_limit_auth_middleware, rate_limit_middleware};
