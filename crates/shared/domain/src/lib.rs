//! Domain layer - Core business entities and value objects.
//!
//! This crate contains pure domain logic with no infrastructure dependencies.
//! All types here are shared across microservices via the proto crate.

pub mod constants;
pub mod error;
pub mod password;
pub mod user;

pub use constants::*;
pub use error::{DomainError, DomainResult};
pub use password::Password;
pub use user::{CreateUser, UpdateUser, User, UserResponse, UserRole};
