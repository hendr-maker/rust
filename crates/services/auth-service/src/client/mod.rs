//! gRPC clients for calling other services.

mod user_client;

pub use user_client::{UserClient, UserServiceClient};
