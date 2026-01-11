//! Infrastructure layer - database and external services.

mod db;
pub mod migrations;

pub use db::Database;
pub use migrations::Migrator;
