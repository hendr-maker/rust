//! User service - Handles user-related business logic.
//!
//! SOLID (SRP): Handles user-related use cases only.
//! DDD: Orchestrates domain operations via Unit of Work.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::User;
use crate::errors::{AppError, AppResult};
use crate::infra::UnitOfWork;

/// User service trait for dependency injection.
///
/// By default, operations exclude soft-deleted users.
/// Use `*_with_deleted` variants to include them.
#[async_trait]
pub trait UserService: Send + Sync {
    /// Get active user by ID (excludes soft-deleted)
    async fn get_user(&self, id: Uuid) -> AppResult<User>;

    /// Get user by ID including soft-deleted
    async fn get_user_with_deleted(&self, id: Uuid) -> AppResult<User>;

    /// List all active users (excludes soft-deleted)
    async fn list_users(&self) -> AppResult<Vec<User>>;

    /// List all users including soft-deleted
    async fn list_users_with_deleted(&self) -> AppResult<Vec<User>>;

    /// List only soft-deleted users
    async fn list_deleted_users(&self) -> AppResult<Vec<User>>;

    /// Update user details (only active users)
    async fn update_user(&self, id: Uuid, name: Option<String>, role: Option<String>) -> AppResult<User>;

    /// Soft delete user (sets deleted_at timestamp)
    async fn delete_user(&self, id: Uuid) -> AppResult<()>;

    /// Permanently delete user from database (hard delete)
    async fn hard_delete_user(&self, id: Uuid) -> AppResult<()>;

    /// Restore a soft-deleted user
    async fn restore_user(&self, id: Uuid) -> AppResult<User>;
}

/// Concrete implementation of UserService using Unit of Work.
pub struct UserManager<U: UnitOfWork> {
    uow: Arc<U>,
}

impl<U: UnitOfWork> UserManager<U> {
    /// Create new user service instance with Unit of Work
    pub fn new(uow: Arc<U>) -> Self {
        Self { uow }
    }
}

#[async_trait]
impl<U: UnitOfWork> UserService for UserManager<U> {
    async fn get_user(&self, id: Uuid) -> AppResult<User> {
        self.uow
            .users()
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)
    }

    async fn get_user_with_deleted(&self, id: Uuid) -> AppResult<User> {
        self.uow
            .users()
            .find_by_id_with_deleted(id)
            .await?
            .ok_or(AppError::NotFound)
    }

    async fn list_users(&self) -> AppResult<Vec<User>> {
        self.uow.users().list().await
    }

    async fn list_users_with_deleted(&self) -> AppResult<Vec<User>> {
        self.uow.users().list_with_deleted().await
    }

    async fn list_deleted_users(&self) -> AppResult<Vec<User>> {
        self.uow.users().list_deleted().await
    }

    async fn update_user(&self, id: Uuid, name: Option<String>, role: Option<String>) -> AppResult<User> {
        self.uow.users().update(id, name, role).await
    }

    async fn delete_user(&self, id: Uuid) -> AppResult<()> {
        self.uow.users().delete(id).await
    }

    async fn hard_delete_user(&self, id: Uuid) -> AppResult<()> {
        self.uow.users().hard_delete(id).await
    }

    async fn restore_user(&self, id: Uuid) -> AppResult<User> {
        self.uow.users().restore(id).await
    }
}
