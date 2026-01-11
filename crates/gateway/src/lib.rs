//! API Gateway Library
//!
//! This crate provides the HTTP REST API that translates requests to gRPC calls.

pub mod clients;
pub mod config;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod openapi;
pub mod routes;
pub mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use tower_http::trace::TraceLayer;
use tracing::info;

use crate::clients::{AuthClient, UserClient};
use crate::config::GatewayConfig;
use crate::middleware::Cache;
use crate::routes::create_router;
use crate::state::AppState;

/// Run the gateway as an embedded component (for combined binary).
pub async fn run_embedded(
    host: &str,
    port: u16,
    auth_port: u16,
    user_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = GatewayConfig::from_env();
    config.auth_service_url = format!("http://{}:{}", host, auth_port);
    config.user_service_url = format!("http://{}:{}", host, user_port);

    run_server_with_config(host, port, config).await
}

/// Run the HTTP server with the given configuration.
async fn run_server_with_config(
    host: &str,
    port: u16,
    config: GatewayConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create gRPC clients
    let auth_client = Arc::new(AuthClient::connect(&config.auth_service_url).await?);
    let user_client = Arc::new(UserClient::connect(&config.user_service_url).await?);

    // Create cache
    let cache = Arc::new(Cache::connect(&config.redis_url).await?);

    // Create app state
    let state = AppState::new(auth_client, user_client, cache, config);

    // Build router
    let app = create_router(state).layer(TraceLayer::new_for_http());

    // Build address
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!("Gateway listening on {}", addr);

    // Run server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
