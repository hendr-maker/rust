//! Auth Service Library
//!
//! This crate provides authentication functionality via gRPC.
//! It communicates with user-service to manage users and handles JWT tokens.

pub mod client;
pub mod config;
pub mod grpc;
pub mod service;

use std::net::SocketAddr;
use std::sync::Arc;

use tonic::transport::Server;
use tracing::info;

use crate::client::UserClient;
use crate::config::AuthServiceConfig;
use crate::grpc::AuthGrpcService;
use crate::service::Authenticator;

/// Run the auth service as an embedded component (for combined binary).
pub async fn run_embedded(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let config = AuthServiceConfig::from_env();
    run_server_with_config(host, port, config).await
}

/// Run the gRPC server with the given configuration.
async fn run_server_with_config(
    host: &str,
    port: u16,
    config: AuthServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create gRPC client to user-service
    let user_client = UserClient::connect(&config.user_service_url).await?;

    // Create auth service
    let auth_service = Arc::new(Authenticator::new(
        Arc::new(user_client),
        config.jwt_secret.clone(),
        config.jwt_expiration_hours,
    ));

    // Create gRPC service
    let grpc_service = AuthGrpcService::new(auth_service);

    // Build address
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    info!("Auth service listening on {}", addr);

    // Run server
    Server::builder()
        .add_service(proto::AuthServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
