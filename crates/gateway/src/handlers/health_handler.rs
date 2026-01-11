//! Health check handlers.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde::Serialize;

use crate::state::AppState;

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub services: ServiceStatus,
}

/// Individual service status.
#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub redis: ServiceHealth,
}

/// Service health with optional error message.
#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Create health routes.
pub fn health_routes() -> Router<AppState> {
    Router::new().route("/", get(health_check))
}

/// Health check endpoint - verifies Redis connectivity.
pub async fn health_check(State(state): State<AppState>) -> Response {
    // Check Redis connectivity
    let redis_health = match state.cache.get::<String>("health_check").await {
        Ok(_) => ServiceHealth {
            status: "healthy".to_string(),
            error: None,
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            error: Some(e.to_string()),
        },
    };

    let all_healthy = redis_health.status == "healthy";

    let response = HealthResponse {
        status: if all_healthy { "healthy" } else { "degraded" }.to_string(),
        services: ServiceStatus {
            redis: redis_health,
        },
    };

    if all_healthy {
        (StatusCode::OK, Json(response)).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response()
    }
}
