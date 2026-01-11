//! Common utilities shared across all microservices.
//!
//! This crate provides:
//! - Unified error handling for HTTP and gRPC
//! - Configuration structures
//! - Common middleware helpers

pub mod config;
pub mod error;

pub use config::*;
pub use error::{AppError, AppResult, OptionExt};
