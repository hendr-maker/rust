//! Infrastructure layer - External systems integration
//!
//! This module handles all external system concerns:
//! - Database connections and repositories
//! - External API clients
//! - Caching systems (Redis)
//! - Message queues
//! - Unit of Work for transaction management

pub mod cache;
pub mod db;
pub mod repositories;
pub mod unit_of_work;

pub use cache::{Cache, LockGuard, SemaphorePermit};
pub use db::{Database, Migrator};
pub use repositories::{UserRepository, UserStore};
pub use unit_of_work::{TransactionContext, TxUserRepository, UnitOfWork, Persistence};

#[cfg(any(test, feature = "test-utils"))]
pub use repositories::MockUserRepository;
