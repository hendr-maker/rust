//! Migrate command - Database migration management.

use crate::cli::args::{MigrateAction, MigrateArgs};
use crate::config::Config;
use crate::errors::{AppError, AppResult};
use crate::infra::Database;

/// Execute the migrate command
pub async fn execute(args: MigrateArgs, config: Config) -> AppResult<()> {
    tracing::info!("Running migration command...");

    // Connect without auto-running migrations for manual control
    let db = Database::connect_without_migrations(&config)
        .await
        .map_err(|e| AppError::internal(format!("Database connection failed: {}", e)))?;

    match args.action {
        MigrateAction::Up => {
            tracing::info!("Running pending migrations...");
            db.run_migrations()
                .await
                .map_err(|e| AppError::internal(e.to_string()))?;
            tracing::info!("Migrations completed successfully");
        }
        MigrateAction::Down => {
            tracing::info!("Rolling back last migration...");
            db.rollback_migration()
                .await
                .map_err(|e| AppError::internal(e.to_string()))?;
            tracing::info!("Rollback completed successfully");
        }
        MigrateAction::Status => {
            tracing::info!("Checking migration status...");
            let status = db
                .migration_status()
                .await
                .map_err(|e| AppError::internal(e.to_string()))?;
            for (name, applied) in status {
                let status_str = if applied { "applied" } else { "pending" };
                println!("{}: {}", name, status_str);
            }
        }
        MigrateAction::Fresh => {
            tracing::warn!("Resetting database and running all migrations...");
            db.fresh_migrations()
                .await
                .map_err(|e| AppError::internal(e.to_string()))?;
            tracing::info!("Fresh migrations completed successfully");
        }
    }

    Ok(())
}
