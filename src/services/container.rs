//! Service Container - Centralized service access with parallel execution support.
//!
//! SOLID (SRP): Manages service lifecycle and access.
//! SOLID (DIP): Depends on service traits, not implementations.
//!
//! Features:
//! - Centralized access to all application services
//! - Thread-safe concurrent access via Arc
//! - Parallel execution utilities for independent operations
//! - Compatible with async/await and tokio runtime

use std::future::Future;
use std::sync::Arc;

use super::{AuthService, UserService};
use crate::config::Config;
use crate::errors::AppResult;
use crate::infra::Persistence;

#[cfg(any(test, feature = "test-utils"))]
use mockall::automock;

/// Service container trait for dependency injection.
///
/// Provides centralized access to all application services.
#[cfg_attr(any(test, feature = "test-utils"), automock)]
pub trait ServiceContainer: Send + Sync {
    /// Get authentication service
    fn auth(&self) -> Arc<dyn AuthService>;

    /// Get user service
    fn users(&self) -> Arc<dyn UserService>;
}

/// Concrete implementation of ServiceContainer
pub struct Services {
    auth_service: Arc<dyn AuthService>,
    user_service: Arc<dyn UserService>,
}

impl Services {
    /// Create a new service container with all services initialized
    pub fn new(
        auth_service: Arc<dyn AuthService>,
        user_service: Arc<dyn UserService>,
    ) -> Self {
        Self {
            auth_service,
            user_service,
        }
    }

    /// Create service container from database connection and config
    pub fn from_connection(
        db: sea_orm::DatabaseConnection,
        config: Config,
    ) -> Self {
        use super::{Authenticator, UserManager};

        let uow = Arc::new(Persistence::new(db));
        let auth_service = Arc::new(Authenticator::new(uow.clone(), config));
        let user_service = Arc::new(UserManager::new(uow.clone()));

        Self {
            auth_service,
            user_service,
        }
    }
}

impl ServiceContainer for Services {
    fn auth(&self) -> Arc<dyn AuthService> {
        self.auth_service.clone()
    }

    fn users(&self) -> Arc<dyn UserService> {
        self.user_service.clone()
    }
}

/// Parallel execution utilities for running independent operations concurrently.
///
/// These functions leverage tokio's async runtime to execute multiple
/// independent operations in parallel, improving throughput.
pub mod parallel {
    use super::*;
    use tokio::try_join;

    /// Execute two independent async operations in parallel.
    ///
    /// Both operations run concurrently and the function returns when both complete.
    /// If either operation fails, the error is returned immediately.
    ///
    /// # Example
    /// ```ignore
    /// let (user, token) = parallel::join2(
    ///     services.users().get_user(id),
    ///     services.auth().verify_token(token),
    /// ).await?;
    /// ```
    pub async fn join2<F1, F2, T1, T2>(f1: F1, f2: F2) -> AppResult<(T1, T2)>
    where
        F1: Future<Output = AppResult<T1>>,
        F2: Future<Output = AppResult<T2>>,
    {
        try_join!(f1, f2)
    }

    /// Execute three independent async operations in parallel.
    pub async fn join3<F1, F2, F3, T1, T2, T3>(
        f1: F1,
        f2: F2,
        f3: F3,
    ) -> AppResult<(T1, T2, T3)>
    where
        F1: Future<Output = AppResult<T1>>,
        F2: Future<Output = AppResult<T2>>,
        F3: Future<Output = AppResult<T3>>,
    {
        try_join!(f1, f2, f3)
    }

    /// Execute four independent async operations in parallel.
    pub async fn join4<F1, F2, F3, F4, T1, T2, T3, T4>(
        f1: F1,
        f2: F2,
        f3: F3,
        f4: F4,
    ) -> AppResult<(T1, T2, T3, T4)>
    where
        F1: Future<Output = AppResult<T1>>,
        F2: Future<Output = AppResult<T2>>,
        F3: Future<Output = AppResult<T3>>,
        F4: Future<Output = AppResult<T4>>,
    {
        try_join!(f1, f2, f3, f4)
    }

    /// Execute a collection of homogeneous async operations in parallel.
    ///
    /// All operations must return the same type. Results are returned in
    /// the same order as the input futures.
    ///
    /// # Example
    /// ```ignore
    /// let user_ids = vec![id1, id2, id3];
    /// let futures: Vec<_> = user_ids
    ///     .iter()
    ///     .map(|id| services.users().get_user(*id))
    ///     .collect();
    /// let users = parallel::join_all(futures).await?;
    /// ```
    pub async fn join_all<F, T>(futures: Vec<F>) -> AppResult<Vec<T>>
    where
        F: Future<Output = AppResult<T>>,
    {
        let results = futures::future::join_all(futures).await;
        results.into_iter().collect()
    }

    /// Execute operations in parallel with a concurrency limit.
    ///
    /// Useful when you have many operations but want to limit
    /// concurrent database connections or API calls.
    ///
    /// # Example
    /// ```ignore
    /// let user_ids: Vec<Uuid> = get_many_ids();
    /// let users = parallel::join_all_limited(
    ///     user_ids.into_iter().map(|id| services.users().get_user(id)),
    ///     10, // Max 10 concurrent operations
    /// ).await?;
    /// ```
    pub async fn join_all_limited<F, T, I>(futures: I, limit: usize) -> AppResult<Vec<T>>
    where
        F: Future<Output = AppResult<T>>,
        I: IntoIterator<Item = F>,
    {
        use futures::stream::{self, StreamExt, TryStreamExt};

        stream::iter(futures)
            .map(Ok)
            .try_buffer_unordered(limit)
            .try_collect()
            .await
    }

    /// Race multiple operations, returning the first successful result.
    ///
    /// Useful for implementing fallback strategies or timeouts.
    ///
    /// # Example
    /// ```ignore
    /// let result = parallel::race(vec![
    ///     cache.get_user(id),
    ///     db.get_user(id),
    /// ]).await?;
    /// ```
    pub async fn race<F, T>(futures: Vec<F>) -> AppResult<T>
    where
        F: Future<Output = AppResult<T>> + Unpin,
    {
        use futures::future::select_all;
        use crate::errors::AppError;

        if futures.is_empty() {
            return Err(AppError::internal("No futures provided to race"));
        }

        let (result, _index, _remaining) = select_all(futures).await;
        result
    }
}

/// Batch operations for efficient bulk processing.
pub mod batch {
    use super::*;
    use crate::errors::AppError;

    /// Process items in batches with parallel execution within each batch.
    ///
    /// Takes ownership of items. For cloneable items that need to be retained,
    /// clone them before passing.
    ///
    /// # Arguments
    /// * `items` - Items to process (ownership transferred)
    /// * `batch_size` - Number of items per batch (must be > 0)
    /// * `processor` - Async function to process each item
    ///
    /// # Example
    /// ```ignore
    /// let user_ids: Vec<Uuid> = get_all_user_ids();
    /// let users = batch::process(
    ///     user_ids,
    ///     100, // Process 100 at a time
    ///     |id| services.users().get_user(id),
    /// ).await?;
    /// ```
    pub async fn process<T, R, F, Fut>(
        items: Vec<T>,
        batch_size: usize,
        processor: F,
    ) -> AppResult<Vec<R>>
    where
        T: Send,
        R: Send,
        F: Fn(T) -> Fut + Send + Sync,
        Fut: Future<Output = AppResult<R>> + Send,
    {
        // Validate batch_size to prevent infinite loop
        if batch_size == 0 {
            return Err(AppError::validation("batch_size must be greater than 0"));
        }

        let capacity = items.len();
        let mut results = Vec::with_capacity(capacity);
        let mut remaining = items;

        while !remaining.is_empty() {
            let drain_count = std::cmp::min(batch_size, remaining.len());
            let chunk: Vec<T> = remaining.drain(..drain_count).collect();

            let chunk_futures: Vec<_> = chunk
                .into_iter()
                .map(|item| processor(item))
                .collect();

            let chunk_results = parallel::join_all(chunk_futures).await?;
            results.extend(chunk_results);
        }

        Ok(results)
    }

    /// Process items in batches with a concurrency limit per batch.
    ///
    /// Similar to `process` but limits concurrent operations within each batch.
    ///
    /// # Arguments
    /// * `items` - Items to process
    /// * `batch_size` - Number of items per batch (must be > 0)
    /// * `concurrency` - Max concurrent operations per batch
    /// * `processor` - Async function to process each item
    pub async fn process_limited<T, R, F, Fut>(
        items: Vec<T>,
        batch_size: usize,
        concurrency: usize,
        processor: F,
    ) -> AppResult<Vec<R>>
    where
        T: Send,
        R: Send,
        F: Fn(T) -> Fut + Send + Sync,
        Fut: Future<Output = AppResult<R>> + Send,
    {
        if batch_size == 0 {
            return Err(AppError::validation("batch_size must be greater than 0"));
        }
        if concurrency == 0 {
            return Err(AppError::validation("concurrency must be greater than 0"));
        }

        let capacity = items.len();
        let mut results = Vec::with_capacity(capacity);
        let mut remaining = items;

        while !remaining.is_empty() {
            let drain_count = std::cmp::min(batch_size, remaining.len());
            let chunk: Vec<T> = remaining.drain(..drain_count).collect();

            let chunk_results = parallel::join_all_limited(
                chunk.into_iter().map(|item| processor(item)),
                concurrency,
            )
            .await?;
            results.extend(chunk_results);
        }

        Ok(results)
    }
}

/// Pipeline pattern for chaining async operations.
pub struct Pipeline<T> {
    value: T,
}

impl<T> Pipeline<T> {
    /// Create a new pipeline with an initial value
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// Transform the value using an async function
    pub async fn then<F, Fut, U>(self, f: F) -> AppResult<Pipeline<U>>
    where
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = AppResult<U>>,
    {
        let result = f(self.value).await?;
        Ok(Pipeline::new(result))
    }

    /// Transform the value using a sync function
    pub fn map<F, U>(self, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> U,
    {
        Pipeline::new(f(self.value))
    }

    /// Extract the final value
    pub fn finish(self) -> T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_join2() {
        async fn op1() -> AppResult<i32> {
            Ok(1)
        }
        async fn op2() -> AppResult<i32> {
            Ok(2)
        }

        let (a, b) = parallel::join2(op1(), op2()).await.unwrap();
        assert_eq!(a, 1);
        assert_eq!(b, 2);
    }

    #[tokio::test]
    async fn test_parallel_join_all() {
        let futures: Vec<_> = (0..5).map(|i| async move { Ok(i) as AppResult<i32> }).collect();
        let results = parallel::join_all(futures).await.unwrap();
        assert_eq!(results, vec![0, 1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_pipeline() {
        let result = Pipeline::new(5)
            .map(|x| x * 2)
            .then(|x| async move { Ok(x + 1) as AppResult<i32> })
            .await
            .unwrap()
            .finish();

        assert_eq!(result, 11);
    }
}
