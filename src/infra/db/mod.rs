//! Database connection and initialization.

use sea_orm::{ConnectionTrait, Database as SeaDatabase, DatabaseConnection, DbErr, Statement};
use sea_orm_migration::MigratorTrait;

use crate::config::Config;

pub mod migrations;

pub use migrations::Migrator;

/// Database wrapper for connection management
#[derive(Clone)]
pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    /// Initialize database connection and run migrations.
    ///
    /// # Panics
    /// Panics if database connection fails.
    pub async fn connect(config: &Config) -> Self {
        let connection = SeaDatabase::connect(&config.database_url)
            .await
            .expect("Failed to connect to database");

        // Run pending migrations
        if let Err(e) = Migrator::up(&connection, None).await {
            tracing::error!("Failed to run migrations: {}", e);
            panic!("Failed to run migrations: {}", e);
        }

        tracing::info!("Database connected and migrations applied");

        Self { connection }
    }

    /// Connect without running migrations (for CLI commands).
    pub async fn connect_without_migrations(config: &Config) -> Result<Self, DbErr> {
        let connection = SeaDatabase::connect(&config.database_url).await?;
        Ok(Self { connection })
    }

    /// Get a reference to the database connection.
    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    /// Get a clone of the database connection.
    pub fn get_connection(&self) -> DatabaseConnection {
        self.connection.clone()
    }

    /// Run pending migrations.
    pub async fn run_migrations(&self) -> Result<(), DbErr> {
        Migrator::up(&self.connection, None).await
    }

    /// Rollback the last migration.
    pub async fn rollback_migration(&self) -> Result<(), DbErr> {
        Migrator::down(&self.connection, Some(1)).await
    }

    /// Get migration status (list all migrations with applied status).
    pub async fn migration_status(&self) -> Result<Vec<(String, bool)>, DbErr> {
        use sea_orm::{EntityTrait, QueryOrder};
        use sea_orm_migration::seaql_migrations;

        // Get applied migrations from database
        let applied: std::collections::HashSet<String> = seaql_migrations::Entity::find()
            .order_by_asc(seaql_migrations::Column::Version)
            .all(&self.connection)
            .await?
            .into_iter()
            .map(|m| m.version)
            .collect();

        // Map all defined migrations with their applied status
        let migrations: Vec<(String, bool)> = Migrator::migrations()
            .iter()
            .map(|m| {
                let name = m.name().to_string();
                let is_applied = applied.contains(&name);
                (name, is_applied)
            })
            .collect();

        Ok(migrations)
    }

    /// Reset database and run all migrations fresh.
    pub async fn fresh_migrations(&self) -> Result<(), DbErr> {
        Migrator::fresh(&self.connection).await
    }

    /// Check database connectivity by executing a simple query.
    pub async fn ping(&self) -> Result<(), DbErr> {
        self.connection
            .execute(Statement::from_string(
                self.connection.get_database_backend(),
                "SELECT 1".to_string(),
            ))
            .await?;
        Ok(())
    }
}
