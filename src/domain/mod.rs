//! Domain layer - Core business entities and logic
//!
//! This module contains the core domain models that represent
//! business concepts independent of infrastructure concerns.
//!
//! DDD: Domain layer has NO external dependencies (except error types).
//! Contains: Entities, Value Objects, Domain Services.

pub mod password;
pub mod user;

pub use password::Password;
pub use user::{CreateUser, UpdateUser, User, UserResponse, UserRole};
