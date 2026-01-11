//! Route configuration.

use axum::{middleware, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{auth_routes, health_routes, user_routes};
use crate::middleware::{auth_middleware, rate_limit_auth_middleware, rate_limit_middleware};
use crate::openapi::ApiDoc;
use crate::state::AppState;

/// Create the main router with all routes.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check (no auth, no rate limit)
        .nest("/health", health_routes())
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Auth routes (no auth required, stricter rate limit)
        .nest(
            "/auth",
            auth_routes().route_layer(middleware::from_fn_with_state(
                state.clone(),
                rate_limit_auth_middleware,
            )),
        )
        // User routes (auth required, general rate limit)
        .nest(
            "/users",
            user_routes()
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    rate_limit_middleware,
                )),
        )
        .with_state(state)
}
