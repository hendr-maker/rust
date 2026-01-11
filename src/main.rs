//! Rust API Starter - Application entry point
//!
//! CLI-based entry point that dispatches to various commands.

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rust_api_starter::{
    cli::{Cli, Commands},
    commands,
    config::Config,
};

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize tracing (verbose mode sets debug level)
    init_tracing(cli.verbose);

    // Load configuration
    let config = Config::from_env();
    tracing::debug!("Configuration loaded");

    // Execute command
    let result = match cli.command {
        Commands::Serve(args) => commands::serve::execute(args, config).await,
        Commands::Migrate(args) => commands::migrate::execute(args, config).await,
        Commands::Jobs(args) => commands::jobs::execute(args, config).await,
        Commands::Generate(args) => commands::generate::execute(args).await,
    };

    // Handle errors
    if let Err(e) = result {
        tracing::error!("Command failed: {}", e);
        std::process::exit(1);
    }
}

/// Initialize tracing subscriber
fn init_tracing(verbose: bool) {
    let filter = if verbose {
        "debug".to_string()
    } else {
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(filter))
        .init();
}
