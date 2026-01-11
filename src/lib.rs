//! Rust API Starter - A production-ready API template
//!
//! This crate provides a clean architecture foundation for building
//! REST APIs with Axum, following DDD, SOLID, and DRY principles.
//!
//! # Architecture Layers
//!
//! - **cli**: Command-line interface
//! - **commands**: CLI command implementations
//! - **config**: Application configuration and constants
//! - **domain**: Core business entities and logic
//! - **services**: Application use cases and business logic
//! - **infra**: Infrastructure concerns (database, external APIs)
//! - **api**: HTTP handlers, middleware, and routes
//! - **types**: Shared types (pagination, responses)
//! - **utils**: Utility functions and helpers
//! - **errors**: Centralized error handling
//!
//! # CLI Usage
//!
//! ```bash
//! # Start the server
//! cargo run -- serve
//!
//! # Run migrations
//! cargo run -- migrate up
//!
//! # Generate a new entity
//! cargo run -- generate entity product
//! ```

pub mod api;
pub mod cli;
pub mod commands;
pub mod config;
pub mod domain;
pub mod errors;
pub mod infra;
pub mod jobs;
pub mod services;
pub mod types;
pub mod utils;

// Re-export commonly used types at crate root
pub use api::AppState;
pub use config::Config;
pub use domain::{Password, User, UserRole};
pub use errors::{AppError, AppResult};
pub use infra::Cache;
