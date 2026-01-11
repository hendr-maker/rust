//! CLI module - Command-line interface for the application.
//!
//! Provides commands for:
//! - `serve` - Start the HTTP server
//! - `migrate` - Database migrations
//! - `jobs` - Background job management
//! - `generate` - Code generation

pub mod args;

pub use args::{Cli, Commands};
