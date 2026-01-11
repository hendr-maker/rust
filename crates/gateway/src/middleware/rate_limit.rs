//! Rate limiting middleware.

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{HeaderValue, Request},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

use crate::state::AppState;

/// Rate limit middleware for general endpoints.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let max_requests = state.config.rate_limit_requests;
    let window_seconds = state.config.rate_limit_window_seconds;
    rate_limit_internal(state, connect_info, request, next, max_requests, window_seconds).await
}

/// Rate limit middleware for auth endpoints (stricter).
pub async fn rate_limit_auth_middleware(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let max_requests = state.config.rate_limit_auth_requests;
    let window_seconds = state.config.rate_limit_auth_window_seconds;
    rate_limit_internal(state, connect_info, request, next, max_requests, window_seconds).await
}

async fn rate_limit_internal(
    state: AppState,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request<Body>,
    next: Next,
    max_requests: u64,
    window_seconds: u64,
) -> Response {
    // Get client IP
    let ip = get_client_ip(&request, connect_info);
    let identifier = format!("{}:{}", request.uri().path(), ip);

    // Check rate limit
    let (count, allowed) = match state
        .cache
        .check_rate_limit(&identifier, max_requests, window_seconds)
        .await
    {
        Ok(result) => result,
        Err(_) => {
            // Fail closed: deny on error (security best practice)
            return rate_limit_exceeded_response(max_requests, window_seconds);
        }
    };

    if !allowed {
        return rate_limit_exceeded_response(max_requests, window_seconds);
    }

    // Add rate limit headers
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        "X-RateLimit-Limit",
        HeaderValue::from_str(&max_requests.to_string()).unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        HeaderValue::from_str(&(max_requests.saturating_sub(count)).to_string()).unwrap(),
    );

    response
}

fn get_client_ip(request: &Request<Body>, connect_info: Option<ConnectInfo<SocketAddr>>) -> String {
    // Try X-Forwarded-For header first
    if let Some(forwarded) = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
    {
        if let Some(ip) = forwarded.split(',').next() {
            return ip.trim().to_string();
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = request
        .headers()
        .get("X-Real-IP")
        .and_then(|h| h.to_str().ok())
    {
        return real_ip.to_string();
    }

    // Fall back to connection socket address
    connect_info
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn rate_limit_exceeded_response(max_requests: u64, window_seconds: u64) -> Response {
    let mut response = (
        axum::http::StatusCode::TOO_MANY_REQUESTS,
        "Too many requests. Please try again later.",
    )
        .into_response();

    let headers = response.headers_mut();
    headers.insert(
        "Retry-After",
        HeaderValue::from_str(&window_seconds.to_string()).unwrap(),
    );
    headers.insert("X-RateLimit-Remaining", HeaderValue::from_static("0"));
    headers.insert(
        "X-RateLimit-Limit",
        HeaderValue::from_str(&max_requests.to_string()).unwrap(),
    );

    response
}
