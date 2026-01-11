//! Redis cache for user data and rate limiting.

use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, warn};
use uuid::Uuid;

use common::AppResult;
use domain::User;

/// Default cache TTL in seconds (1 hour)
const DEFAULT_CACHE_TTL: u64 = 3600;

/// Cache key prefix for user data
const CACHE_PREFIX_USER: &str = "user:";

/// Cache key prefix for rate limiting
const CACHE_PREFIX_RATE_LIMIT: &str = "rate_limit:";

/// Redis cache wrapper.
pub struct Cache {
    conn: ConnectionManager,
}

impl Cache {
    /// Connect to Redis.
    pub async fn connect(url: &str) -> Result<Self, RedisError> {
        debug!("Connecting to Redis at {}", url);
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    // =========================================================================
    // Generic Cache Operations
    // =========================================================================

    /// Get a value from cache.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> AppResult<Option<T>> {
        let mut conn = self.conn.clone();
        let result: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| {
                warn!("Redis get error for key {}: {}", key, e);
                common::AppError::internal(format!("Cache error: {}", e))
            })?;

        match result {
            Some(json) => {
                match serde_json::from_str(&json) {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => {
                        warn!("Failed to deserialize cached value for key {}: {}", key, e);
                        Ok(None) // Treat deserialization errors as cache miss
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache with TTL.
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> AppResult<()> {
        self.set_with_ttl(key, value, DEFAULT_CACHE_TTL).await
    }

    /// Set a value in cache with custom TTL.
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> AppResult<()> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(value)
            .map_err(|e| common::AppError::internal(format!("Serialization error: {}", e)))?;
        conn.set_ex::<_, _, ()>(key, json, ttl_seconds)
            .await
            .map_err(|e| {
                warn!("Redis set error for key {}: {}", key, e);
                common::AppError::internal(format!("Cache error: {}", e))
            })?;
        Ok(())
    }

    /// Delete a value from cache.
    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.conn.clone();
        conn.del::<_, ()>(key).await.map_err(|e| {
            warn!("Redis delete error for key {}: {}", key, e);
            common::AppError::internal(format!("Cache error: {}", e))
        })?;
        Ok(())
    }

    // =========================================================================
    // User Cache Operations
    // =========================================================================

    /// Get cached user by ID.
    pub async fn get_user(&self, id: &Uuid) -> AppResult<Option<User>> {
        let key = format!("{}{}", CACHE_PREFIX_USER, id);
        self.get(&key).await
    }

    /// Cache a user.
    pub async fn set_user(&self, user: &User) -> AppResult<()> {
        let key = format!("{}{}", CACHE_PREFIX_USER, user.id);
        self.set(&key, user).await
    }

    /// Invalidate user cache.
    pub async fn invalidate_user(&self, id: &Uuid) -> AppResult<()> {
        let key = format!("{}{}", CACHE_PREFIX_USER, id);
        self.delete(&key).await
    }

    // =========================================================================
    // Rate Limiting
    // =========================================================================

    /// Check rate limit and increment counter.
    /// Returns (current_count, allowed).
    pub async fn check_rate_limit(
        &self,
        identifier: &str,
        max_requests: u64,
        window_seconds: u64,
    ) -> AppResult<(u64, bool)> {
        let key = format!("{}{}", CACHE_PREFIX_RATE_LIMIT, identifier);
        let mut conn = self.conn.clone();

        // Try to increment, or set if doesn't exist
        let count: u64 = conn.incr(&key, 1).await.unwrap_or(1);

        // Set expiry on first request
        if count == 1 {
            let _: () = conn.expire(&key, window_seconds as i64).await.unwrap_or(());
        }

        let allowed = count <= max_requests;
        Ok((count, allowed))
    }

    /// Get TTL for rate limit key.
    pub async fn get_rate_limit_ttl(&self, identifier: &str) -> AppResult<Option<i64>> {
        let key = format!("{}{}", CACHE_PREFIX_RATE_LIMIT, identifier);
        let mut conn = self.conn.clone();
        let ttl: i64 = conn.ttl(&key).await.unwrap_or(-2);
        if ttl < 0 {
            Ok(None)
        } else {
            Ok(Some(ttl))
        }
    }
}
