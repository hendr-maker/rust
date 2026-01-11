//! SeaORM entity definitions
//!
//! These are database-specific entities separate from domain models.

pub mod user;

// Re-exports for public API convenience
#[allow(unused_imports)]
pub use user::{ActiveModel as UserActiveModel, Entity as UserEntity, Model as UserModel};
