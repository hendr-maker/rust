//! gRPC clients for calling microservices.

mod auth_client;
mod user_client;

pub use auth_client::{AuthClient, TokenResponse};
pub use user_client::UserClient;
