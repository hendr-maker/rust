//! Unit of Work pattern implementation.
//!
//! SOLID (SRP): Manages transaction lifecycle and repository access.
//! DDD: Coordinates operations across multiple aggregates atomically.
//!
//! The Unit of Work pattern:
//! - Centralizes access to all repositories
//! - Manages database transactions (begin, commit, rollback)
//! - Ensures consistency across multiple repository operations
//! - Provides atomic operations for complex business workflows

use async_trait::async_trait;
use sea_orm::{
    AccessMode, DatabaseConnection, DatabaseTransaction, IsolationLevel, TransactionTrait,
};
use std::sync::Arc;

use super::repositories::{UserRepository, UserStore};
use crate::errors::{AppError, AppResult};

/// Unit of Work trait for dependency injection.
///
/// Provides centralized access to all repositories and transaction management.
/// Note: This trait is not mockable directly due to generic methods.
/// For testing, mock at the service level or use integration tests.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Get user repository
    fn users(&self) -> Arc<dyn UserRepository>;

    /// Execute a closure within a transaction.
    ///
    /// The transaction is automatically committed on success or rolled back on error.
    /// Uses ReadCommitted isolation level by default for balanced consistency/performance.
    async fn transaction<F, T>(&self, f: F) -> AppResult<T>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = AppResult<T>> + Send + 'a>,
            > + Send,
        T: Send;

    /// Execute a closure within a transaction with serializable isolation.
    ///
    /// Use this for operations requiring the strongest consistency guarantees.
    async fn transaction_serializable<F, T>(&self, f: F) -> AppResult<T>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = AppResult<T>> + Send + 'a>,
            > + Send,
        T: Send;
}

/// Transaction context providing repository access within a transaction.
///
/// All repository operations performed through this context are part
/// of the same database transaction. The context borrows the transaction
/// to ensure proper lifetime management.
pub struct TransactionContext<'a> {
    txn: &'a DatabaseTransaction,
}

impl<'a> TransactionContext<'a> {
    /// Create a new transaction context
    fn new(txn: &'a DatabaseTransaction) -> Self {
        Self { txn }
    }

    /// Get user repository for this transaction
    pub fn users(&self) -> TxUserRepository<'_> {
        TxUserRepository::new(self.txn)
    }
}

/// Concrete implementation of UnitOfWork
pub struct Persistence {
    db: DatabaseConnection,
    user_repo: Arc<UserStore>,
}

impl Persistence {
    /// Create new UnitOfWork instance
    pub fn new(db: DatabaseConnection) -> Self {
        let user_repo = Arc::new(UserStore::new(db.clone()));
        Self { db, user_repo }
    }

    /// Internal transaction execution with configurable isolation level
    async fn execute_transaction<F, T>(&self, isolation: IsolationLevel, f: F) -> AppResult<T>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = AppResult<T>> + Send + 'a>,
            > + Send,
        T: Send,
    {
        // Begin transaction
        let txn = self
            .db
            .begin_with_config(Some(isolation), Some(AccessMode::ReadWrite))
            .await
            .map_err(AppError::from)?;

        // Create context with borrowed transaction
        let ctx = TransactionContext::new(&txn);

        // Execute the closure
        match f(ctx).await {
            Ok(result) => {
                // Commit on success - txn is owned, so this always works
                txn.commit().await.map_err(AppError::from)?;
                Ok(result)
            }
            Err(e) => {
                // Rollback on error
                if let Err(rollback_err) = txn.rollback().await {
                    tracing::error!("Transaction rollback failed: {}", rollback_err);
                }
                Err(e)
            }
        }
    }
}

#[async_trait]
impl UnitOfWork for Persistence {
    fn users(&self) -> Arc<dyn UserRepository> {
        self.user_repo.clone()
    }

    async fn transaction<F, T>(&self, f: F) -> AppResult<T>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = AppResult<T>> + Send + 'a>,
            > + Send,
        T: Send,
    {
        // Use ReadCommitted for balanced consistency/performance
        self.execute_transaction(IsolationLevel::ReadCommitted, f).await
    }

    async fn transaction_serializable<F, T>(&self, f: F) -> AppResult<T>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = AppResult<T>> + Send + 'a>,
            > + Send,
        T: Send,
    {
        self.execute_transaction(IsolationLevel::Serializable, f).await
    }
}

/// Transaction-aware user repository with soft delete support.
///
/// Executes all operations within the provided transaction.
/// Uses borrowed reference to ensure transaction outlives repository operations.
/// By default, query methods exclude soft-deleted records.
pub struct TxUserRepository<'a> {
    txn: &'a DatabaseTransaction,
}

impl<'a> TxUserRepository<'a> {
    /// Create new transaction-aware repository
    fn new(txn: &'a DatabaseTransaction) -> Self {
        Self { txn }
    }

    /// Find active user by ID (excludes soft-deleted)
    pub async fn find_by_id(&self, id: uuid::Uuid) -> AppResult<Option<crate::domain::User>> {
        use super::repositories::entities::user::{self, Entity as UserEntity};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let result = UserEntity::find_by_id(id)
            .filter(user::Column::DeletedAt.is_null())
            .one(self.txn)
            .await
            .map_err(AppError::from)?;

        Ok(result.map(crate::domain::User::from))
    }

    /// Find user by ID including soft-deleted
    pub async fn find_by_id_with_deleted(&self, id: uuid::Uuid) -> AppResult<Option<crate::domain::User>> {
        use super::repositories::entities::user::Entity as UserEntity;
        use sea_orm::EntityTrait;

        let result = UserEntity::find_by_id(id)
            .one(self.txn)
            .await
            .map_err(AppError::from)?;

        Ok(result.map(crate::domain::User::from))
    }

    /// Find active user by email (excludes soft-deleted)
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<crate::domain::User>> {
        use super::repositories::entities::user::{self, Entity as UserEntity};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let result = UserEntity::find()
            .filter(user::Column::Email.eq(email))
            .filter(user::Column::DeletedAt.is_null())
            .one(self.txn)
            .await
            .map_err(AppError::from)?;

        Ok(result.map(crate::domain::User::from))
    }

    /// Find user by email including soft-deleted
    pub async fn find_by_email_with_deleted(&self, email: &str) -> AppResult<Option<crate::domain::User>> {
        use super::repositories::entities::user::{self, Entity as UserEntity};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let result = UserEntity::find()
            .filter(user::Column::Email.eq(email))
            .one(self.txn)
            .await
            .map_err(AppError::from)?;

        Ok(result.map(crate::domain::User::from))
    }

    /// Create a new user
    pub async fn create(
        &self,
        email: String,
        password_hash: String,
        name: String,
    ) -> AppResult<crate::domain::User> {
        use super::repositories::entities::user::ActiveModel;
        use crate::config::ROLE_USER;
        use sea_orm::{ActiveModelTrait, Set};

        let now = chrono::Utc::now();
        let active_model = ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            email: Set(email),
            password_hash: Set(password_hash),
            name: Set(name),
            role: Set(ROLE_USER.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        let model = active_model
            .insert(self.txn)
            .await
            .map_err(AppError::from)?;

        Ok(crate::domain::User::from(model))
    }

    /// Update user fields (only active users)
    pub async fn update(
        &self,
        id: uuid::Uuid,
        name: Option<String>,
        role: Option<String>,
    ) -> AppResult<crate::domain::User> {
        use super::repositories::entities::user::{self, ActiveModel, Entity as UserEntity};
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

        let user = UserEntity::find_by_id(id)
            .filter(user::Column::DeletedAt.is_null())
            .one(self.txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: ActiveModel = user.into();

        if let Some(name) = name {
            active.name = Set(name);
        }
        if let Some(role) = role {
            active.role = Set(role);
        }
        active.updated_at = Set(chrono::Utc::now());

        let model = active.update(self.txn).await.map_err(AppError::from)?;

        Ok(crate::domain::User::from(model))
    }

    /// Soft delete user by ID (sets deleted_at timestamp)
    pub async fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        use super::repositories::entities::user::{self, ActiveModel, Entity as UserEntity};
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

        let user = UserEntity::find_by_id(id)
            .filter(user::Column::DeletedAt.is_null())
            .one(self.txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: ActiveModel = user.into();
        let now = chrono::Utc::now();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);

        active.update(self.txn).await.map_err(AppError::from)?;
        Ok(())
    }

    /// Permanently delete user from database (hard delete)
    pub async fn hard_delete(&self, id: uuid::Uuid) -> AppResult<()> {
        use super::repositories::entities::user::Entity as UserEntity;
        use sea_orm::EntityTrait;

        let result = UserEntity::delete_by_id(id)
            .exec(self.txn)
            .await
            .map_err(AppError::from)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// Restore a soft-deleted user
    pub async fn restore(&self, id: uuid::Uuid) -> AppResult<crate::domain::User> {
        use super::repositories::entities::user::{self, ActiveModel, Entity as UserEntity};
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

        let user = UserEntity::find_by_id(id)
            .filter(user::Column::DeletedAt.is_not_null())
            .one(self.txn)
            .await?
            .ok_or_else(|| AppError::validation("User is not deleted or does not exist"))?;

        let mut active: ActiveModel = user.into();
        active.deleted_at = Set(None);
        active.updated_at = Set(chrono::Utc::now());

        let model = active.update(self.txn).await.map_err(AppError::from)?;
        Ok(crate::domain::User::from(model))
    }

    /// List active users (excludes soft-deleted)
    pub async fn list(&self) -> AppResult<Vec<crate::domain::User>> {
        use super::repositories::entities::user::{self, Entity as UserEntity};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let models = UserEntity::find()
            .filter(user::Column::DeletedAt.is_null())
            .all(self.txn)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map(crate::domain::User::from).collect())
    }

    /// List all users including soft-deleted
    pub async fn list_with_deleted(&self) -> AppResult<Vec<crate::domain::User>> {
        use super::repositories::entities::user::Entity as UserEntity;
        use sea_orm::EntityTrait;

        let models = UserEntity::find()
            .all(self.txn)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map(crate::domain::User::from).collect())
    }
}

/// Simpler API for executing transactional operations.
///
/// This helper macro reduces boilerplate when using transactions.
#[macro_export]
macro_rules! with_transaction {
    ($uow:expr, |$ctx:ident| $body:expr) => {
        $uow.transaction(|$ctx| Box::pin(async move { $body })).await
    };
}
