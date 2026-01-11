//! API layer - HTTP handlers and middleware
//!
//! This module contains all HTTP-related concerns:
//! - Request handlers
//! - Middleware (authentication, logging)
//! - Custom extractors
//! - Route definitions

pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod openapi;
pub mod routes;
pub mod state;

pub use openapi::ApiDoc;
pub use routes::create_router;
pub use state::AppState;
