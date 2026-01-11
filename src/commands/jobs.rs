//! Jobs command - Background job management.
//!
//! Provides CLI commands to manage background jobs:
//! - `work`: Start the job worker process
//! - `list`: Show pending/failed jobs
//! - `clear`: Remove failed jobs from the queue
//!
//! ## Usage
//!
//! ```bash
//! # Start the job worker
//! cargo run -- jobs work
//!
//! # List job queue status
//! cargo run -- jobs list
//!
//! # Clear failed jobs
//! cargo run -- jobs clear
//! ```

use crate::cli::args::{JobsAction, JobsArgs};
use crate::config::Config;
use crate::errors::{AppError, AppResult};

/// Execute the jobs command
pub async fn execute(args: JobsArgs, config: Config) -> AppResult<()> {
    match args.action {
        JobsAction::Work => run_worker(&config).await,
        JobsAction::List => list_jobs(&config).await,
        JobsAction::Clear => clear_failed_jobs(&config).await,
    }
}

/// Start the background job worker
///
/// Connects to the database and starts processing jobs from the queue.
/// Uses apalis with PostgreSQL storage for job persistence.
async fn run_worker(config: &Config) -> AppResult<()> {
    use apalis::prelude::*;
    use apalis_sql::postgres::PostgresStorage;
    use apalis_sql::sqlx::postgres::PgPoolOptions;

    use crate::jobs::{email_job_handler, EmailJob};

    tracing::info!("Connecting to database for job worker...");

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .map_err(|e| AppError::internal(format!("Failed to connect to database: {}", e)))?;

    // Run migrations for apalis tables first (associated function on pool)
    PostgresStorage::setup(&pool)
        .await
        .map_err(|e| AppError::internal(format!("Failed to setup job storage: {}", e)))?;

    // Initialize PostgreSQL storage for email jobs
    let email_storage: PostgresStorage<EmailJob> = PostgresStorage::new(pool);

    tracing::info!("Job worker started. Press Ctrl+C to stop.");

    // Build and run the worker
    let worker = WorkerBuilder::new("email-worker")
        .backend(email_storage)
        .build_fn(email_job_handler);

    // Run with graceful shutdown on Ctrl+C
    let monitor = Monitor::new().register(worker);

    tokio::select! {
        result = monitor.run() => {
            if let Err(e) = result {
                tracing::error!("Worker error: {}", e);
                return Err(AppError::internal(format!("Worker failed: {}", e)));
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal, stopping worker...");
        }
    }

    tracing::info!("Job worker stopped.");
    Ok(())
}

/// List pending and failed jobs
///
/// Queries the apalis job tables and displays status counts.
async fn list_jobs(config: &Config) -> AppResult<()> {
    use sea_orm::{ConnectionTrait, Database, Statement};

    tracing::info!("Connecting to database...");

    let db = Database::connect(&config.database_url)
        .await
        .map_err(|e| AppError::internal(format!("Failed to connect to database: {}", e)))?;

    // Check if apalis schema exists
    let result = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT EXISTS(SELECT 1 FROM information_schema.schemata WHERE schema_name = 'apalis') as exists".to_string(),
        ))
        .await
        .map_err(|e| AppError::internal(format!("Query failed: {}", e)))?;

    let schema_exists = result
        .and_then(|r| r.try_get::<bool>("", "exists").ok())
        .unwrap_or(false);

    if !schema_exists {
        println!("\n=== Job Queue Status ===");
        println!("Job queue not initialized.");
        println!("Run 'jobs work' first to create the queue tables.");
        println!("========================\n");
        return Ok(());
    }

    // Query job counts
    let rows = db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT status::text as status, COUNT(*)::bigint as count FROM apalis.jobs GROUP BY status".to_string(),
        ))
        .await
        .unwrap_or_default();

    let mut pending = 0i64;
    let mut running = 0i64;
    let mut failed = 0i64;
    let mut done = 0i64;

    for row in rows {
        if let (Ok(status), Ok(count)) = (
            row.try_get::<String>("", "status"),
            row.try_get::<i64>("", "count"),
        ) {
            match status.as_str() {
                "Pending" => pending = count,
                "Running" => running = count,
                "Failed" => failed = count,
                "Done" => done = count,
                _ => {}
            }
        }
    }

    println!("\n=== Job Queue Status ===");
    println!("Pending:  {}", pending);
    println!("Running:  {}", running);
    println!("Failed:   {}", failed);
    println!("Done:     {}", done);
    println!("========================\n");

    Ok(())
}

/// Clear failed jobs from the queue
async fn clear_failed_jobs(config: &Config) -> AppResult<()> {
    use sea_orm::{ConnectionTrait, Database, Statement};

    tracing::info!("Connecting to database...");

    let db = Database::connect(&config.database_url)
        .await
        .map_err(|e| AppError::internal(format!("Failed to connect to database: {}", e)))?;

    // Check if apalis schema exists
    let result = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT EXISTS(SELECT 1 FROM information_schema.schemata WHERE schema_name = 'apalis') as exists".to_string(),
        ))
        .await
        .map_err(|e| AppError::internal(format!("Query failed: {}", e)))?;

    let schema_exists = result
        .and_then(|r| r.try_get::<bool>("", "exists").ok())
        .unwrap_or(false);

    if !schema_exists {
        println!("Job queue not initialized. Nothing to clear.");
        return Ok(());
    }

    // Delete failed jobs
    let result = db
        .execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "DELETE FROM apalis.jobs WHERE status = 'Failed'".to_string(),
        ))
        .await
        .map_err(|e| AppError::internal(format!("Failed to clear jobs: {}", e)))?;

    let count = result.rows_affected();
    println!("Cleared {} failed job(s) from the queue.", count);

    Ok(())
}
