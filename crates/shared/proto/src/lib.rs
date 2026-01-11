//! gRPC protocol buffer definitions.
//!
//! This crate contains the generated gRPC service definitions for:
//! - AuthService: User authentication (register, login, verify)
//! - UserService: User management (CRUD, soft delete)

/// Authentication service definitions.
pub mod auth {
    tonic::include_proto!("auth");
}

/// User service definitions.
pub mod user {
    tonic::include_proto!("user");
}

// Re-export commonly used items
pub use auth::auth_service_client::AuthServiceClient;
pub use auth::auth_service_server::{AuthService, AuthServiceServer};
pub use user::user_service_client::UserServiceClient;
pub use user::user_service_server::{UserService, UserServiceServer};
