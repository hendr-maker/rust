//! User service - Handles user-related business logic.
//!
//! SOLID (SRP): Handles user-related use cases only.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use common::{AppError, AppResult};
use domain::User;

use crate::repository::UserRepository;

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

    /// Get active user by email (excludes soft-deleted)
    async fn get_user_by_email(&self, email: &str) -> AppResult<User>;

    /// Get user by email including soft-deleted
    async fn get_user_by_email_with_deleted(&self, email: &str) -> AppResult<User>;

    /// List all active users (excludes soft-deleted)
    async fn list_users(&self) -> AppResult<Vec<User>>;

    /// List all users including soft-deleted
    async fn list_users_with_deleted(&self) -> AppResult<Vec<User>>;

    /// List only soft-deleted users
    async fn list_deleted_users(&self) -> AppResult<Vec<User>>;

    /// Create a new user (internal use - password already hashed)
    async fn create_user(&self, email: String, password_hash: String, name: String)
        -> AppResult<User>;

    /// Update user details (only active users)
    async fn update_user(
        &self,
        id: Uuid,
        name: Option<String>,
        role: Option<String>,
    ) -> AppResult<User>;

    /// Soft delete user (sets deleted_at timestamp)
    async fn delete_user(&self, id: Uuid) -> AppResult<()>;

    /// Permanently delete user from database (hard delete)
    async fn hard_delete_user(&self, id: Uuid) -> AppResult<()>;

    /// Restore a soft-deleted user
    async fn restore_user(&self, id: Uuid) -> AppResult<User>;
}

/// Concrete implementation of UserService using repository.
pub struct UserManager {
    repo: Arc<dyn UserRepository>,
}

impl UserManager {
    /// Create new user service instance with repository
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl UserService for UserManager {
    async fn get_user(&self, id: Uuid) -> AppResult<User> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)
    }

    async fn get_user_with_deleted(&self, id: Uuid) -> AppResult<User> {
        self.repo
            .find_by_id_with_deleted(id)
            .await?
            .ok_or(AppError::NotFound)
    }

    async fn get_user_by_email(&self, email: &str) -> AppResult<User> {
        self.repo
            .find_by_email(email)
            .await?
            .ok_or(AppError::NotFound)
    }

    async fn get_user_by_email_with_deleted(&self, email: &str) -> AppResult<User> {
        self.repo
            .find_by_email_with_deleted(email)
            .await?
            .ok_or(AppError::NotFound)
    }

    async fn list_users(&self) -> AppResult<Vec<User>> {
        self.repo.list().await
    }

    async fn list_users_with_deleted(&self) -> AppResult<Vec<User>> {
        self.repo.list_with_deleted().await
    }

    async fn list_deleted_users(&self) -> AppResult<Vec<User>> {
        self.repo.list_deleted().await
    }

    async fn create_user(
        &self,
        email: String,
        password_hash: String,
        name: String,
    ) -> AppResult<User> {
        // Check if email already exists
        if self.repo.find_by_email_with_deleted(&email).await?.is_some() {
            return Err(AppError::conflict("Email"));
        }

        self.repo.create(email, password_hash, name).await
    }

    async fn update_user(
        &self,
        id: Uuid,
        name: Option<String>,
        role: Option<String>,
    ) -> AppResult<User> {
        self.repo.update(id, name, role).await
    }

    async fn delete_user(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete(id).await
    }

    async fn hard_delete_user(&self, id: Uuid) -> AppResult<()> {
        self.repo.hard_delete(id).await
    }

    async fn restore_user(&self, id: Uuid) -> AppResult<User> {
        self.repo.restore(id).await
    }
}
