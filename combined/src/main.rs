//! Combined binary for development - runs all services in one process.

use clap::{Parser, Subcommand};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "rust-api")]
#[command(about = "Combined microservices binary for development")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all services in a single process (development mode)
    Serve {
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long, default_value = "3000")]
        gateway_port: u16,
        #[arg(long, default_value = "50051")]
        auth_port: u16,
        #[arg(long, default_value = "50052")]
        user_port: u16,
    },
    /// Run database migrations for all services
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },
}

#[derive(Subcommand, Clone, Copy)]
enum MigrateAction {
    /// Run pending migrations
    Up,
    /// Rollback last migration
    Down,
    /// Show migration status
    Status,
    /// Reset database and run all migrations
    Fresh,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            host,
            gateway_port,
            auth_port,
            user_port,
        } => {
            info!("Starting combined services in development mode");
            info!("  Gateway:      http://{}:{}", host, gateway_port);
            info!("  Auth service: http://{}:{}", host, auth_port);
            info!("  User service: http://{}:{}", host, user_port);

            // Spawn user-service first (it owns the database)
            let user_host = host.clone();
            let user_handle = tokio::spawn(async move {
                if let Err(e) = user_service_lib::run_embedded(&user_host, user_port).await {
                    error!("User service failed: {}", e);
                }
            });

            // Wait a moment for user-service to start
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Spawn auth-service (depends on user-service)
            let auth_host = host.clone();
            let auth_handle = tokio::spawn(async move {
                if let Err(e) = auth_service_lib::run_embedded(&auth_host, auth_port).await {
                    error!("Auth service failed: {}", e);
                }
            });

            // Wait a moment for auth-service to start
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Spawn gateway (depends on both services)
            let gateway_host = host.clone();
            let gateway_handle = tokio::spawn(async move {
                if let Err(e) =
                    gateway_lib::run_embedded(&gateway_host, gateway_port, auth_port, user_port)
                        .await
                {
                    error!("Gateway failed: {}", e);
                }
            });

            // Wait for any service to exit (which would indicate an error)
            tokio::select! {
                _ = user_handle => {
                    error!("User service exited unexpectedly");
                }
                _ = auth_handle => {
                    error!("Auth service exited unexpectedly");
                }
                _ = gateway_handle => {
                    error!("Gateway exited unexpectedly");
                }
            }
        }
        Commands::Migrate { action } => {
            let migrate_action = match action {
                MigrateAction::Up => user_service_lib::MigrateAction::Up,
                MigrateAction::Down => user_service_lib::MigrateAction::Down,
                MigrateAction::Status => user_service_lib::MigrateAction::Status,
                MigrateAction::Fresh => user_service_lib::MigrateAction::Fresh,
            };

            // Run migrations for user-service (owns the database)
            user_service_lib::run_migrations(migrate_action).await?;
        }
    }

    Ok(())
}
