//! User Service Library
//!
//! This crate provides user management functionality via gRPC.
//! It can be run as a standalone service or embedded in the combined binary.

pub mod config;
pub mod grpc;
pub mod infra;
pub mod repository;
pub mod service;

use std::net::SocketAddr;
use std::sync::Arc;

use tonic::transport::Server;
use tracing::info;

use crate::config::UserServiceConfig;
use crate::grpc::UserGrpcService;
use crate::infra::Database;
use crate::repository::UserStore;
use crate::service::UserManager;

/// Run the user service as an embedded component (for combined binary).
pub async fn run_embedded(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let config = UserServiceConfig::from_env();
    run_server_with_config(host, port, config).await
}

/// Run migrations (for CLI commands).
pub async fn run_migrations(action: MigrateAction) -> Result<(), Box<dyn std::error::Error>> {
    let config = UserServiceConfig::from_env();
    let db = Database::connect_without_migrations(&config.database_url).await?;

    match action {
        MigrateAction::Up => {
            db.run_migrations().await?;
            info!("Migrations applied successfully");
        }
        MigrateAction::Down => {
            db.rollback_migration().await?;
            info!("Rolled back last migration");
        }
        MigrateAction::Status => {
            let status = db.migration_status().await?;
            for (name, applied) in status {
                let marker = if applied { "[x]" } else { "[ ]" };
                println!("{} {}", marker, name);
            }
        }
        MigrateAction::Fresh => {
            db.fresh_migrations().await?;
            info!("Database reset and migrations applied");
        }
    }

    Ok(())
}

/// Migration action type.
#[derive(Debug, Clone, Copy)]
pub enum MigrateAction {
    Up,
    Down,
    Status,
    Fresh,
}

/// Run the gRPC server with the given configuration.
async fn run_server_with_config(
    host: &str,
    port: u16,
    config: UserServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db = Database::connect(&config.database_url).await?;
    let db_conn = db.get_connection();

    // Create repository and service
    let user_repo = Arc::new(UserStore::new(db_conn));
    let user_service = Arc::new(UserManager::new(user_repo));

    // Create gRPC service
    let grpc_service = UserGrpcService::new(user_service);

    // Build address
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!("User service listening on {}", addr);

    // Run server
    Server::builder()
        .add_service(proto::UserServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
