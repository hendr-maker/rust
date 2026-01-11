//! Application services layer - Use cases and business logic.
//!
//! Services orchestrate domain logic and infrastructure to fulfill
//! application use cases. They depend on abstractions (traits) for
//! dependency inversion.
//!
//! All services use Unit of Work pattern for centralized repository
//! access and transaction management.

mod auth_service;
pub mod container;
mod user_service;

// Service Container
pub use container::{ServiceContainer, Services};

// Service traits and implementations
pub use auth_service::{AuthService, Authenticator, Claims, TokenResponse};
pub use user_service::{UserService, UserManager};

// Parallel execution utilities
pub use container::{batch, parallel, Pipeline};

#[cfg(any(test, feature = "test-utils"))]
pub use container::MockServiceContainer;
