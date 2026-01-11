//! API middleware.

mod auth;
mod rate_limit;

pub use auth::{auth_middleware, require_admin, require_role, CurrentUser};
pub use rate_limit::{rate_limit_auth_middleware, rate_limit_middleware};
