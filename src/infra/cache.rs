//! Redis cache implementation.
//!
//! Provides a type-safe caching layer with connection pooling,
//! distributed locks, and semaphores for concurrency control.

use redis::{aio::ConnectionManager, AsyncCommands, Client, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::config::{
    Config, CACHE_PREFIX_LOCK, CACHE_PREFIX_RATE_LIMIT, CACHE_PREFIX_SEMAPHORE,
    CACHE_PREFIX_SESSION, CACHE_PREFIX_USER, DEFAULT_CACHE_TTL_SECONDS,
    DEFAULT_LOCK_RETRY_DELAY_MS, DEFAULT_LOCK_RETRIES, DEFAULT_LOCK_TTL_SECONDS,
};
use crate::domain::User;
use crate::errors::{AppError, AppResult};

/// Redis cache wrapper with connection pooling.
#[derive(Clone)]
pub struct Cache {
    connection: ConnectionManager,
    default_ttl: u64,
}

impl Cache {
    /// Create a new cache instance and connect to Redis.
    ///
    /// # Panics
    /// Panics if Redis connection fails.
    pub async fn connect(config: &Config) -> Self {
        let client = Client::open(config.redis_url.as_str())
            .expect("Failed to create Redis client");

        let connection = ConnectionManager::new(client)
            .await
            .expect("Failed to connect to Redis");

        tracing::info!("Redis cache connected");

        Self {
            connection,
            default_ttl: DEFAULT_CACHE_TTL_SECONDS,
        }
    }

    /// Try to connect to Redis, returning an error instead of panicking.
    pub async fn try_connect(config: &Config) -> Result<Self, RedisError> {
        let client = Client::open(config.redis_url.as_str())?;
        let connection = ConnectionManager::new(client).await?;

        Ok(Self {
            connection,
            default_ttl: DEFAULT_CACHE_TTL_SECONDS,
        })
    }

    /// Get the connection manager for direct Redis operations.
    pub fn connection(&self) -> ConnectionManager {
        self.connection.clone()
    }

    // =========================================================================
    // Generic Cache Operations
    // =========================================================================

    /// Get a value from cache.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> AppResult<Option<T>> {
        let mut conn = self.connection.clone();
        let value: Option<String> = conn.get(key).await.map_err(cache_error)?;

        match value {
            Some(json) => {
                let parsed = serde_json::from_str(&json).map_err(|e| {
                    AppError::internal(format!("Cache deserialization error: {}", e))
                })?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache with default TTL.
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> AppResult<()> {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Set a value in cache with custom TTL (in seconds).
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> AppResult<()> {
        let mut conn = self.connection.clone();
        let json = serde_json::to_string(value)
            .map_err(|e| AppError::internal(format!("Cache serialization error: {}", e)))?;

        conn.set_ex::<_, _, ()>(key, json, ttl_seconds)
            .await
            .map_err(cache_error)?;

        Ok(())
    }

    /// Delete a value from cache.
    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.connection.clone();
        let _: () = conn.del(key).await.map_err(cache_error)?;
        Ok(())
    }

    /// Check if a key exists in cache.
    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.connection.clone();
        let exists: bool = conn.exists(key).await.map_err(cache_error)?;
        Ok(exists)
    }

    /// Set key expiration time in seconds.
    pub async fn expire(&self, key: &str, seconds: u64) -> AppResult<()> {
        let mut conn = self.connection.clone();
        let _: () = conn.expire(key, seconds as i64).await.map_err(cache_error)?;
        Ok(())
    }

    /// Increment a counter value.
    pub async fn incr(&self, key: &str) -> AppResult<i64> {
        let mut conn = self.connection.clone();
        let value: i64 = conn.incr(key, 1).await.map_err(cache_error)?;
        Ok(value)
    }

    /// Delete all keys matching a pattern.
    /// Uses UNLINK for non-blocking async deletion in Redis.
    pub async fn delete_pattern(&self, pattern: &str) -> AppResult<u64> {
        let mut conn = self.connection.clone();
        let keys: Vec<String> = conn.keys(pattern).await.map_err(cache_error)?;

        if keys.is_empty() {
            return Ok(0);
        }

        let count = keys.len() as u64;

        // Use UNLINK for non-blocking deletion (Redis 4.0+)
        // Falls back to DEL if UNLINK is not available
        let deleted: i64 = redis::cmd("UNLINK")
            .arg(&keys)
            .query_async(&mut conn)
            .await
            .unwrap_or_else(|_| {
                // Fallback: this branch runs sync but only on UNLINK failure
                0
            });

        // If UNLINK failed (returned 0 but we had keys), try batch DEL
        if deleted == 0 && !keys.is_empty() {
            let _: i64 = conn.del(&keys).await.map_err(cache_error)?;
        }

        Ok(count)
    }

    // =========================================================================
    // User Cache Operations
    // =========================================================================

    /// Get cached user by ID.
    pub async fn get_user(&self, user_id: &uuid::Uuid) -> AppResult<Option<User>> {
        let key = format!("{}{}", CACHE_PREFIX_USER, user_id);
        self.get(&key).await
    }

    /// Cache a user.
    pub async fn set_user(&self, user: &User) -> AppResult<()> {
        let key = format!("{}{}", CACHE_PREFIX_USER, user.id);
        self.set(&key, user).await
    }

    /// Invalidate cached user.
    pub async fn invalidate_user(&self, user_id: &uuid::Uuid) -> AppResult<()> {
        let key = format!("{}{}", CACHE_PREFIX_USER, user_id);
        self.delete(&key).await
    }

    // =========================================================================
    // Session Cache Operations
    // =========================================================================

    /// Store session data.
    pub async fn set_session<T: Serialize>(
        &self,
        session_id: &str,
        data: &T,
        ttl_seconds: u64,
    ) -> AppResult<()> {
        let key = format!("{}{}", CACHE_PREFIX_SESSION, session_id);
        self.set_with_ttl(&key, data, ttl_seconds).await
    }

    /// Get session data.
    pub async fn get_session<T: DeserializeOwned>(&self, session_id: &str) -> AppResult<Option<T>> {
        let key = format!("{}{}", CACHE_PREFIX_SESSION, session_id);
        self.get(&key).await
    }

    /// Delete session.
    pub async fn delete_session(&self, session_id: &str) -> AppResult<()> {
        let key = format!("{}{}", CACHE_PREFIX_SESSION, session_id);
        self.delete(&key).await
    }

    // =========================================================================
    // Rate Limiting Operations
    // =========================================================================

    /// Check and increment rate limit counter.
    /// Returns (current_count, is_allowed) tuple.
    pub async fn check_rate_limit(
        &self,
        identifier: &str,
        max_requests: u64,
        window_seconds: u64,
    ) -> AppResult<(u64, bool)> {
        let key = format!("{}{}", CACHE_PREFIX_RATE_LIMIT, identifier);
        let mut conn = self.connection.clone();

        // Check if key exists
        let exists: bool = conn.exists(&key).await.map_err(cache_error)?;

        if !exists {
            // First request in window
            let _: () = conn.set_ex(&key, 1i64, window_seconds)
                .await
                .map_err(cache_error)?;
            return Ok((1, true));
        }

        // Increment counter
        let count: i64 = conn.incr(&key, 1).await.map_err(cache_error)?;
        let count = count as u64;
        let allowed = count <= max_requests;

        Ok((count, allowed))
    }

    /// Get remaining requests in rate limit window.
    pub async fn get_rate_limit_remaining(
        &self,
        identifier: &str,
        max_requests: u64,
    ) -> AppResult<u64> {
        let key = format!("{}{}", CACHE_PREFIX_RATE_LIMIT, identifier);
        let mut conn = self.connection.clone();

        let count: Option<i64> = conn.get(&key).await.map_err(cache_error)?;
        let count = count.unwrap_or(0) as u64;

        Ok(max_requests.saturating_sub(count))
    }

    // =========================================================================
    // Distributed Lock Operations
    // =========================================================================

    /// Acquire a distributed lock with default settings.
    /// Returns a LockGuard that automatically releases the lock when dropped.
    pub async fn acquire_lock(&self, resource: &str) -> AppResult<LockGuard> {
        self.acquire_lock_with_options(
            resource,
            DEFAULT_LOCK_TTL_SECONDS,
            DEFAULT_LOCK_RETRIES,
            DEFAULT_LOCK_RETRY_DELAY_MS,
        )
        .await
    }

    /// Acquire a distributed lock with custom options.
    pub async fn acquire_lock_with_options(
        &self,
        resource: &str,
        ttl_seconds: u64,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> AppResult<LockGuard> {
        let key = format!("{}{}", CACHE_PREFIX_LOCK, resource);
        let lock_id = Uuid::new_v4().to_string();
        let mut conn = self.connection.clone();

        for attempt in 0..=max_retries {
            // Try to acquire lock using SET NX (set if not exists)
            let acquired: bool = redis::cmd("SET")
                .arg(&key)
                .arg(&lock_id)
                .arg("NX")
                .arg("EX")
                .arg(ttl_seconds)
                .query_async(&mut conn)
                .await
                .map(|r: Option<String>| r.is_some())
                .unwrap_or(false);

            if acquired {
                tracing::debug!(resource = %resource, lock_id = %lock_id, "Lock acquired");
                return Ok(LockGuard {
                    cache: Arc::new(self.clone()),
                    key,
                    lock_id,
                    released: false,
                });
            }

            if attempt < max_retries {
                sleep(Duration::from_millis(retry_delay_ms)).await;
            }
        }

        tracing::warn!(resource = %resource, "Failed to acquire lock after retries");
        Err(AppError::internal(format!(
            "Failed to acquire lock for resource: {}",
            resource
        )))
    }

    /// Try to acquire lock without retrying.
    /// Returns None if lock is already held.
    pub async fn try_acquire_lock(&self, resource: &str) -> AppResult<Option<LockGuard>> {
        let key = format!("{}{}", CACHE_PREFIX_LOCK, resource);
        let lock_id = Uuid::new_v4().to_string();
        let mut conn = self.connection.clone();

        let acquired: bool = redis::cmd("SET")
            .arg(&key)
            .arg(&lock_id)
            .arg("NX")
            .arg("EX")
            .arg(DEFAULT_LOCK_TTL_SECONDS)
            .query_async(&mut conn)
            .await
            .map(|r: Option<String>| r.is_some())
            .unwrap_or(false);

        if acquired {
            tracing::debug!(resource = %resource, lock_id = %lock_id, "Lock acquired");
            Ok(Some(LockGuard {
                cache: Arc::new(self.clone()),
                key,
                lock_id,
                released: false,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if a resource is currently locked.
    pub async fn is_locked(&self, resource: &str) -> AppResult<bool> {
        let key = format!("{}{}", CACHE_PREFIX_LOCK, resource);
        self.exists(&key).await
    }

    /// Release a lock (internal use - prefer using LockGuard).
    async fn release_lock(&self, key: &str, lock_id: &str) -> AppResult<bool> {
        let mut conn = self.connection.clone();

        // Use Lua script to atomically check and delete
        // Only delete if the lock_id matches (we own the lock)
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

        let released: i32 = redis::cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(key)
            .arg(lock_id)
            .query_async(&mut conn)
            .await
            .map_err(cache_error)?;

        Ok(released == 1)
    }

    // =========================================================================
    // Semaphore Operations
    // =========================================================================

    /// Acquire a semaphore permit.
    /// Limits concurrent access to a resource to max_permits.
    pub async fn acquire_semaphore(
        &self,
        resource: &str,
        max_permits: u64,
    ) -> AppResult<SemaphorePermit> {
        self.acquire_semaphore_with_options(
            resource,
            max_permits,
            DEFAULT_LOCK_TTL_SECONDS,
            DEFAULT_LOCK_RETRIES,
            DEFAULT_LOCK_RETRY_DELAY_MS,
        )
        .await
    }

    /// Acquire a semaphore permit with custom options.
    pub async fn acquire_semaphore_with_options(
        &self,
        resource: &str,
        max_permits: u64,
        ttl_seconds: u64,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> AppResult<SemaphorePermit> {
        let key = format!("{}{}", CACHE_PREFIX_SEMAPHORE, resource);
        let permit_id = Uuid::new_v4().to_string();
        let mut conn = self.connection.clone();

        // Lua script for atomic semaphore acquisition
        // Checks count and adds permit atomically to prevent race conditions
        let script = r#"
            local current = redis.call("SCARD", KEYS[1])
            if current < tonumber(ARGV[1]) then
                local added = redis.call("SADD", KEYS[1], ARGV[2])
                if added == 1 then
                    redis.call("EXPIRE", KEYS[1], ARGV[3])
                    return current + 1
                end
            end
            return -1
        "#;

        for attempt in 0..=max_retries {
            let result: i64 = redis::cmd("EVAL")
                .arg(script)
                .arg(1)
                .arg(&key)
                .arg(max_permits)
                .arg(&permit_id)
                .arg(ttl_seconds)
                .query_async(&mut conn)
                .await
                .unwrap_or(-1);

            if result >= 0 {
                tracing::debug!(
                    resource = %resource,
                    permit_id = %permit_id,
                    current = result,
                    max = max_permits,
                    "Semaphore permit acquired"
                );

                return Ok(SemaphorePermit {
                    cache: Arc::new(self.clone()),
                    key,
                    permit_id,
                    released: false,
                });
            }

            if attempt < max_retries {
                sleep(Duration::from_millis(retry_delay_ms)).await;
            }
        }

        tracing::warn!(resource = %resource, "Failed to acquire semaphore permit");
        Err(AppError::internal(format!(
            "Failed to acquire semaphore permit for resource: {}",
            resource
        )))
    }

    /// Try to acquire semaphore without retrying.
    pub async fn try_acquire_semaphore(
        &self,
        resource: &str,
        max_permits: u64,
    ) -> AppResult<Option<SemaphorePermit>> {
        let key = format!("{}{}", CACHE_PREFIX_SEMAPHORE, resource);
        let permit_id = Uuid::new_v4().to_string();
        let mut conn = self.connection.clone();

        // Atomic semaphore acquisition using Lua script
        let script = r#"
            local current = redis.call("SCARD", KEYS[1])
            if current < tonumber(ARGV[1]) then
                local added = redis.call("SADD", KEYS[1], ARGV[2])
                if added == 1 then
                    redis.call("EXPIRE", KEYS[1], ARGV[3])
                    return 1
                end
            end
            return 0
        "#;

        let acquired: i64 = redis::cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(&key)
            .arg(max_permits)
            .arg(&permit_id)
            .arg(DEFAULT_LOCK_TTL_SECONDS)
            .query_async(&mut conn)
            .await
            .map_err(cache_error)?;

        if acquired == 1 {
            Ok(Some(SemaphorePermit {
                cache: Arc::new(self.clone()),
                key,
                permit_id,
                released: false,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get current semaphore count.
    pub async fn semaphore_count(&self, resource: &str) -> AppResult<u64> {
        let key = format!("{}{}", CACHE_PREFIX_SEMAPHORE, resource);
        let mut conn = self.connection.clone();
        let count: i64 = conn.scard(&key).await.map_err(cache_error)?;
        Ok(count as u64)
    }

    /// Release a semaphore permit (internal use - prefer using SemaphorePermit).
    async fn release_semaphore(&self, key: &str, permit_id: &str) -> AppResult<bool> {
        let mut conn = self.connection.clone();
        let removed: i64 = conn.srem(key, permit_id).await.map_err(cache_error)?;
        Ok(removed == 1)
    }
}

// =============================================================================
// Lock Guard (RAII)
// =============================================================================

/// RAII guard for distributed locks.
/// Automatically releases the lock when dropped.
pub struct LockGuard {
    cache: Arc<Cache>,
    key: String,
    lock_id: String,
    released: bool,
}

impl LockGuard {
    /// Manually release the lock early.
    pub async fn release(mut self) -> AppResult<()> {
        self.do_release().await
    }

    /// Extend the lock TTL.
    pub async fn extend(&self, ttl_seconds: u64) -> AppResult<bool> {
        let mut conn = self.cache.connection.clone();

        // Only extend if we still own the lock
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("EXPIRE", KEYS[1], ARGV[2])
            else
                return 0
            end
        "#;

        let extended: i32 = redis::cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(&self.key)
            .arg(&self.lock_id)
            .arg(ttl_seconds)
            .query_async(&mut conn)
            .await
            .map_err(cache_error)?;

        Ok(extended == 1)
    }

    async fn do_release(&mut self) -> AppResult<()> {
        if !self.released {
            self.released = true;
            let released = self.cache.release_lock(&self.key, &self.lock_id).await?;
            if released {
                tracing::debug!(key = %self.key, "Lock released");
            }
        }
        Ok(())
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        if !self.released {
            let cache = self.cache.clone();
            let key = self.key.clone();
            let lock_id = self.lock_id.clone();

            // Spawn a task to release the lock asynchronously
            tokio::spawn(async move {
                if let Err(e) = cache.release_lock(&key, &lock_id).await {
                    tracing::error!(key = %key, error = %e, "Failed to release lock on drop");
                } else {
                    tracing::debug!(key = %key, "Lock released on drop");
                }
            });
        }
    }
}

// =============================================================================
// Semaphore Permit (RAII)
// =============================================================================

/// RAII guard for semaphore permits.
/// Automatically releases the permit when dropped.
pub struct SemaphorePermit {
    cache: Arc<Cache>,
    key: String,
    permit_id: String,
    released: bool,
}

impl SemaphorePermit {
    /// Manually release the permit early.
    pub async fn release(mut self) -> AppResult<()> {
        self.do_release().await
    }

    async fn do_release(&mut self) -> AppResult<()> {
        if !self.released {
            self.released = true;
            let released = self.cache.release_semaphore(&self.key, &self.permit_id).await?;
            if released {
                tracing::debug!(key = %self.key, "Semaphore permit released");
            }
        }
        Ok(())
    }
}

impl Drop for SemaphorePermit {
    fn drop(&mut self) {
        if !self.released {
            let cache = self.cache.clone();
            let key = self.key.clone();
            let permit_id = self.permit_id.clone();

            tokio::spawn(async move {
                if let Err(e) = cache.release_semaphore(&key, &permit_id).await {
                    tracing::error!(key = %key, error = %e, "Failed to release semaphore on drop");
                } else {
                    tracing::debug!(key = %key, "Semaphore permit released on drop");
                }
            });
        }
    }
}

/// Convert Redis error to AppError.
fn cache_error(e: RedisError) -> AppError {
    tracing::error!("Redis error: {}", e);
    AppError::internal(format!("Cache error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_prefixes() {
        assert_eq!(CACHE_PREFIX_USER, "user:");
        assert_eq!(CACHE_PREFIX_SESSION, "session:");
        assert_eq!(CACHE_PREFIX_RATE_LIMIT, "rate_limit:");
        assert_eq!(CACHE_PREFIX_LOCK, "lock:");
        assert_eq!(CACHE_PREFIX_SEMAPHORE, "semaphore:");
    }

    #[test]
    fn test_lock_defaults() {
        assert_eq!(DEFAULT_LOCK_TTL_SECONDS, 30);
        assert_eq!(DEFAULT_LOCK_RETRIES, 10);
        assert_eq!(DEFAULT_LOCK_RETRY_DELAY_MS, 100);
    }
}
