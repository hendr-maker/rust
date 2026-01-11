//! CLI argument definitions.
//!
//! Uses clap derive macros for type-safe argument parsing.

use clap::{Parser, Subcommand};

/// Rust API Starter - Production-ready API with Clean Architecture
#[derive(Parser, Debug)]
#[command(name = "rust-api-starter")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Config file path
    #[arg(short, long, global = true, env = "CONFIG_PATH")]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the HTTP server
    Serve(ServeArgs),

    /// Run database migrations
    Migrate(MigrateArgs),

    /// Manage background jobs
    Jobs(JobsArgs),

    /// Generate project components
    Generate(GenerateArgs),
}

/// Arguments for the serve command
#[derive(Parser, Debug)]
pub struct ServeArgs {
    /// Host to bind to
    #[arg(short = 'H', long, default_value = "0.0.0.0", env = "SERVER_HOST")]
    pub host: String,

    /// Port to listen on
    #[arg(short, long, default_value = "3000", env = "SERVER_PORT")]
    pub port: u16,
}

/// Arguments for the migrate command
#[derive(Parser, Debug)]
pub struct MigrateArgs {
    #[command(subcommand)]
    pub action: MigrateAction,
}

/// Migration actions
#[derive(Subcommand, Debug)]
pub enum MigrateAction {
    /// Run pending migrations
    Up,
    /// Rollback last migration
    Down,
    /// Show migration status
    Status,
    /// Reset and re-run all migrations
    Fresh,
}

/// Arguments for the jobs command
#[derive(Parser, Debug)]
pub struct JobsArgs {
    #[command(subcommand)]
    pub action: JobsAction,
}

/// Job management actions
#[derive(Subcommand, Debug)]
pub enum JobsAction {
    /// Start background job worker
    Work,
    /// List pending jobs
    List,
    /// Clear failed jobs
    Clear,
}

/// Arguments for the generate command
#[derive(Parser, Debug)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub component: GenerateComponent,
}

/// Components that can be generated
#[derive(Subcommand, Debug)]
pub enum GenerateComponent {
    /// Generate a new entity
    Entity {
        /// Entity name (e.g., "product")
        name: String,
    },
    /// Generate a new migration
    Migration {
        /// Migration name (e.g., "create_products_table")
        name: String,
    },
    /// Generate a new service
    Service {
        /// Service name (e.g., "payment")
        name: String,
    },
}
