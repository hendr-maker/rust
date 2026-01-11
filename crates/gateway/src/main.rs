//! API Gateway - HTTP REST API for the microservices.

use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use gateway_lib::config::GatewayConfig;

#[derive(Parser)]
#[command(name = "gateway")]
#[command(about = "API Gateway for microservices")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve {
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        #[arg(long, default_value = "3000")]
        port: u16,
    },
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
    let config = GatewayConfig::from_env();

    match cli.command {
        Commands::Serve { host, port } => {
            // For standalone mode, use default ports from config
            gateway_lib::run_embedded(&host, port, config.auth_port(), config.user_port()).await?;
        }
    }

    Ok(())
}
