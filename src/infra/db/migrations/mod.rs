//! Database migrations.
//!
//! Each migration is a separate module following SeaORM conventions.
//! Migration names follow the pattern: m{YYYYMMDD}_{NNNNNN}_{description}

use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users_table;
mod m20240102_000001_add_soft_delete;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_users_table::Migration),
            Box::new(m20240102_000001_add_soft_delete::Migration),
        ]
    }
}
