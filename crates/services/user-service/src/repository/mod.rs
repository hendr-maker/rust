//! Repository layer for data access.

pub mod entities;
mod user_repository;

pub use user_repository::{UserRepository, UserStore};
