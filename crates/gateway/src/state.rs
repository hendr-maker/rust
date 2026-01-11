//! Application state for dependency injection.

use std::sync::Arc;

use crate::clients::{AuthClient, UserClient};
use crate::config::GatewayConfig;
use crate::middleware::Cache;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub auth_client: Arc<AuthClient>,
    pub user_client: Arc<UserClient>,
    pub cache: Arc<Cache>,
    pub config: GatewayConfig,
}

impl AppState {
    /// Create new app state.
    pub fn new(
        auth_client: Arc<AuthClient>,
        user_client: Arc<UserClient>,
        cache: Arc<Cache>,
        config: GatewayConfig,
    ) -> Self {
        Self {
            auth_client,
            user_client,
            cache,
            config,
        }
    }
}
