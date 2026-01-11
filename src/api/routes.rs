//! Application route configuration.

use axum::{extract::State, http::StatusCode, middleware, response::Json, routing::get, Router};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::handlers::{auth_routes, user_routes};
use super::middleware::{auth_middleware, rate_limit_auth_middleware, rate_limit_middleware};
use super::openapi::ApiDoc;
use super::AppState;

/// Create the application router with all routes configured
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check endpoints (no rate limiting)
        .route("/", get(root))
        .route("/health", get(health))
        // OpenAPI Swagger UI documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Public authentication routes (stricter rate limiting)
        .nest(
            "/auth",
            auth_routes().route_layer(middleware::from_fn_with_state(
                state.clone(),
                rate_limit_auth_middleware,
            )),
        )
        // Protected user routes (require JWT + general rate limiting)
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
        // Global middleware
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Root endpoint
async fn root() -> &'static str {
    "Welcome to Rust API Starter"
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    services: ServiceHealth,
}

/// Individual service health status
#[derive(Serialize)]
struct ServiceHealth {
    database: ServiceStatus,
    redis: ServiceStatus,
}

/// Service status
#[derive(Serialize)]
struct ServiceStatus {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Health check endpoint with database and Redis connectivity check
async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    // Check database health
    let db_status = match state.database.ping().await {
        Ok(_) => ServiceStatus {
            status: "healthy",
            error: None,
        },
        Err(e) => ServiceStatus {
            status: "unhealthy",
            error: Some(e.to_string()),
        },
    };

    // Check Redis health
    let redis_status = match state.cache.exists("health:ping").await {
        Ok(_) => ServiceStatus {
            status: "healthy",
            error: None,
        },
        Err(e) => ServiceStatus {
            status: "unhealthy",
            error: Some(e.to_string()),
        },
    };

    let all_healthy = db_status.status == "healthy" && redis_status.status == "healthy";

    let response = HealthResponse {
        status: if all_healthy { "healthy" } else { "degraded" },
        services: ServiceHealth {
            database: db_status,
            redis: redis_status,
        },
    };

    let status_code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(response))
}
