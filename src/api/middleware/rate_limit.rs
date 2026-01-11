//! Rate limiting middleware using Redis cache.

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

use crate::api::AppState;
use crate::config::{
    RATE_LIMIT_AUTH_REQUESTS, RATE_LIMIT_AUTH_WINDOW_SECONDS, RATE_LIMIT_REQUESTS,
    RATE_LIMIT_WINDOW_SECONDS,
};

/// Rate limit error response
#[derive(Debug)]
pub struct RateLimitError {
    pub retry_after: u64,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Retry-After",
            HeaderValue::from_str(&self.retry_after.to_string()).unwrap(),
        );
        headers.insert(
            "X-RateLimit-Remaining",
            HeaderValue::from_static("0"),
        );

        (
            StatusCode::TOO_MANY_REQUESTS,
            headers,
            "Too many requests. Please try again later.",
        )
            .into_response()
    }
}

/// Extract client identifier for rate limiting.
/// Uses X-Forwarded-For header if behind proxy, otherwise uses connection IP.
fn get_client_identifier(request: &Request) -> String {
    // Try X-Forwarded-For header first (for reverse proxies)
    if let Some(forwarded) = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
    {
        // Take the first IP in the chain (original client)
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

    // Fall back to connection info
    if let Some(connect_info) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        return connect_info.0.ip().to_string();
    }

    // Last resort: unknown
    "unknown".to_string()
}

/// General rate limiting middleware.
/// Limits requests to RATE_LIMIT_REQUESTS per RATE_LIMIT_WINDOW_SECONDS.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    let client_id = get_client_identifier(&request);
    let key = format!("general:{}", client_id);

    let (count, allowed) = match state
        .cache
        .check_rate_limit(&key, RATE_LIMIT_REQUESTS, RATE_LIMIT_WINDOW_SECONDS)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            // SECURITY: Fail closed - deny requests when Redis is unavailable
            // to prevent rate limit bypass attacks
            tracing::error!(error = %e, "Rate limit check failed - denying request");
            return Err(RateLimitError {
                retry_after: RATE_LIMIT_WINDOW_SECONDS,
            });
        }
    };

    if !allowed {
        tracing::warn!(
            client = %client_id,
            count = count,
            "Rate limit exceeded"
        );
        return Err(RateLimitError {
            retry_after: RATE_LIMIT_WINDOW_SECONDS,
        });
    }

    let mut response = next.run(request).await;

    // Add rate limit headers
    let remaining = RATE_LIMIT_REQUESTS.saturating_sub(count);
    response.headers_mut().insert(
        "X-RateLimit-Limit",
        HeaderValue::from_str(&RATE_LIMIT_REQUESTS.to_string()).unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        HeaderValue::from_str(&remaining.to_string()).unwrap(),
    );

    Ok(response)
}

/// Stricter rate limiting for authentication endpoints.
/// Limits requests to RATE_LIMIT_AUTH_REQUESTS per RATE_LIMIT_AUTH_WINDOW_SECONDS.
pub async fn rate_limit_auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    let client_id = get_client_identifier(&request);
    let key = format!("auth:{}", client_id);

    let (count, allowed) = match state
        .cache
        .check_rate_limit(&key, RATE_LIMIT_AUTH_REQUESTS, RATE_LIMIT_AUTH_WINDOW_SECONDS)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            // SECURITY: Fail closed - deny auth requests when Redis is unavailable
            // to prevent brute-force attacks bypassing rate limits
            tracing::error!(error = %e, "Auth rate limit check failed - denying request");
            return Err(RateLimitError {
                retry_after: RATE_LIMIT_AUTH_WINDOW_SECONDS,
            });
        }
    };

    if !allowed {
        tracing::warn!(
            client = %client_id,
            count = count,
            "Auth rate limit exceeded"
        );
        return Err(RateLimitError {
            retry_after: RATE_LIMIT_AUTH_WINDOW_SECONDS,
        });
    }

    let mut response = next.run(request).await;

    // Add rate limit headers
    let remaining = RATE_LIMIT_AUTH_REQUESTS.saturating_sub(count);
    response.headers_mut().insert(
        "X-RateLimit-Limit",
        HeaderValue::from_str(&RATE_LIMIT_AUTH_REQUESTS.to_string()).unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        HeaderValue::from_str(&remaining.to_string()).unwrap(),
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_error_response() {
        let error = RateLimitError { retry_after: 60 };
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
