//! Serve command - Starts the HTTP server.

use std::sync::Arc;

use crate::api::{create_router, AppState};
use crate::cli::args::ServeArgs;
use crate::config::Config;
use crate::errors::{AppError, AppResult};
use crate::infra::{Cache, Database};

/// Execute the serve command
pub async fn execute(args: ServeArgs, config: Config) -> AppResult<()> {
    tracing::info!("Starting server...");

    // Initialize database
    let db = Arc::new(Database::connect(&config).await);
    tracing::info!("Database connected");

    // Initialize Redis cache
    let cache = Arc::new(Cache::connect(&config).await);
    tracing::info!("Redis cache connected");

    // Create application state with centralized service container
    // Uses Unit of Work internally for repository access
    let app_state = AppState::from_config(db, cache, config);

    // Build router
    let app = create_router(app_state);

    // Start server
    let addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::internal(format!("Failed to bind to {}: {}", addr, e)))?;

    tracing::info!("Server running on http://{}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::internal(format!("Server error: {}", e)))?;

    Ok(())
}
