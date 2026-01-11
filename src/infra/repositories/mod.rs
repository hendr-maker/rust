//! Repository layer - Data access abstraction
//!
//! Repositories provide an abstraction over data persistence,
//! following the Repository pattern for clean separation of concerns.

mod base;
pub(crate) mod entities;
mod user_repository;

pub use base::{CrudRepository, DeleteRepository, ReadRepository, WriteRepository};
pub use user_repository::{UserRepository, UserStore};

// Export mock for tests (both unit and integration)
#[cfg(any(test, feature = "test-utils"))]
pub use user_repository::MockUserRepository;
