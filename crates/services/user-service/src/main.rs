//! User Service - gRPC server for user management.

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use user_service_lib::MigrateAction;

#[derive(Parser)]
#[command(name = "user-service")]
#[command(about = "User management microservice")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the gRPC server
    Serve {
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long, default_value = "50052")]
        port: u16,
    },
    /// Database migration commands
    Migrate {
        #[command(subcommand)]
        action: MigrateCommands,
    },
}

#[derive(Subcommand)]
enum MigrateCommands {
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
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { host, port } => {
            user_service_lib::run_embedded(&host, port).await?;
        }
        Commands::Migrate { action } => {
            let migrate_action = match action {
                MigrateCommands::Up => MigrateAction::Up,
                MigrateCommands::Down => MigrateAction::Down,
                MigrateCommands::Status => MigrateAction::Status,
                MigrateCommands::Fresh => MigrateAction::Fresh,
            };
            user_service_lib::run_migrations(migrate_action).await?;
        }
    }

    Ok(())
}
