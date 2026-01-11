//! Application state - Dependency injection container.
//!
//! Provides centralized access to all application services and infrastructure.

use std::sync::Arc;

use crate::infra::{Cache, Database};
use crate::services::{AuthService, ServiceContainer, Services, UserService};

/// Application state containing all services (DI container).
///
/// Use `from_config()` for recommended initialization with full
/// ServiceContainer and UnitOfWork support.
#[derive(Clone)]
pub struct AppState {
    /// Authentication service
    pub auth_service: Arc<dyn AuthService>,
    /// User service
    pub user_service: Arc<dyn UserService>,
    /// Redis cache
    pub cache: Arc<Cache>,
    /// Database connection
    pub database: Arc<Database>,
    /// Internal service container (optional, only with from_config)
    service_container: Option<Arc<Services>>,
}

impl AppState {
    /// Create application state from database connection and config.
    ///
    /// This is the recommended way to create AppState as it uses
    /// the ServiceContainer for centralized service management.
    pub fn from_config(
        database: Arc<Database>,
        cache: Arc<Cache>,
        config: crate::config::Config,
    ) -> Self {
        let container = Arc::new(Services::from_connection(
            database.get_connection(),
            config,
        ));

        Self {
            auth_service: container.auth(),
            user_service: container.users(),
            cache,
            database,
            service_container: Some(container),
        }
    }

    /// Create new application state with manually injected services.
    ///
    /// Note: This method does not provide ServiceContainer access.
    /// Use `from_config()` for full functionality.
    pub fn new(
        auth_service: Arc<dyn AuthService>,
        user_service: Arc<dyn UserService>,
        cache: Arc<Cache>,
        database: Arc<Database>,
    ) -> Self {
        Self {
            auth_service,
            user_service,
            cache,
            database,
            service_container: None,
        }
    }

    /// Get the service container for centralized service access.
    ///
    /// Returns `Some` only if created via `from_config()`.
    pub fn services(&self) -> Option<&Arc<Services>> {
        self.service_container.as_ref()
    }
}
