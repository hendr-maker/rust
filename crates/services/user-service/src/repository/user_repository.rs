//! User repository implementation with soft delete support.

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use super::entities::user::{self, ActiveModel, Entity as UserEntity};
use common::{AppError, AppResult};
use domain::{User, ROLE_USER};

#[cfg(any(test, feature = "test-utils"))]
use mockall::automock;

/// User repository trait for dependency injection.
///
/// By default, all query methods exclude soft-deleted records.
/// Use `*_with_deleted` variants to include them.
#[cfg_attr(any(test, feature = "test-utils"), automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find active user by ID (excludes soft-deleted)
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;

    /// Find user by ID including soft-deleted
    async fn find_by_id_with_deleted(&self, id: Uuid) -> AppResult<Option<User>>;

    /// Find active user by email address (excludes soft-deleted)
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;

    /// Find user by email including soft-deleted
    async fn find_by_email_with_deleted(&self, email: &str) -> AppResult<Option<User>>;

    /// Create a new user
    async fn create(&self, email: String, password_hash: String, name: String) -> AppResult<User>;

    /// Update user fields
    async fn update(&self, id: Uuid, name: Option<String>, role: Option<String>) -> AppResult<User>;

    /// Soft delete user by ID (sets deleted_at timestamp)
    async fn delete(&self, id: Uuid) -> AppResult<()>;

    /// Permanently delete user from database (hard delete)
    async fn hard_delete(&self, id: Uuid) -> AppResult<()>;

    /// Restore a soft-deleted user
    async fn restore(&self, id: Uuid) -> AppResult<User>;

    /// List all active users (excludes soft-deleted)
    async fn list(&self) -> AppResult<Vec<User>>;

    /// List all users including soft-deleted
    async fn list_with_deleted(&self) -> AppResult<Vec<User>>;

    /// List only soft-deleted users
    async fn list_deleted(&self) -> AppResult<Vec<User>>;
}

/// Concrete implementation of UserRepository with soft delete
pub struct UserStore {
    db: DatabaseConnection,
}

impl UserStore {
    /// Create new repository instance
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for UserStore {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let result = UserEntity::find_by_id(id)
            .filter(user::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(result.map(User::from))
    }

    async fn find_by_id_with_deleted(&self, id: Uuid) -> AppResult<Option<User>> {
        let result = UserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(result.map(User::from))
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let result = UserEntity::find()
            .filter(user::Column::Email.eq(email))
            .filter(user::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(result.map(User::from))
    }

    async fn find_by_email_with_deleted(&self, email: &str) -> AppResult<Option<User>> {
        let result = UserEntity::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(result.map(User::from))
    }

    async fn create(&self, email: String, password_hash: String, name: String) -> AppResult<User> {
        let now = chrono::Utc::now();
        let active_model = ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(email),
            password_hash: Set(password_hash),
            name: Set(name),
            role: Set(ROLE_USER.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };

        let model = active_model.insert(&self.db).await.map_err(AppError::from)?;
        Ok(User::from(model))
    }

    async fn update(&self, id: Uuid, name: Option<String>, role: Option<String>) -> AppResult<User> {
        // Only allow updating active (non-deleted) users
        let user = UserEntity::find_by_id(id)
            .filter(user::Column::DeletedAt.is_null())
            .one(&self.db)
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

        let model = active.update(&self.db).await.map_err(AppError::from)?;
        Ok(User::from(model))
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        // Soft delete: set deleted_at timestamp
        let user = UserEntity::find_by_id(id)
            .filter(user::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: ActiveModel = user.into();
        let now = chrono::Utc::now();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);

        active.update(&self.db).await.map_err(AppError::from)?;
        Ok(())
    }

    async fn hard_delete(&self, id: Uuid) -> AppResult<()> {
        let result = UserEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(AppError::from)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    async fn restore(&self, id: Uuid) -> AppResult<User> {
        // Find the soft-deleted user
        let user = UserEntity::find_by_id(id)
            .filter(user::Column::DeletedAt.is_not_null())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::validation("User is not deleted or does not exist"))?;

        let mut active: ActiveModel = user.into();
        active.deleted_at = Set(None);
        active.updated_at = Set(chrono::Utc::now());

        let model = active.update(&self.db).await.map_err(AppError::from)?;
        Ok(User::from(model))
    }

    async fn list(&self) -> AppResult<Vec<User>> {
        let models = UserEntity::find()
            .filter(user::Column::DeletedAt.is_null())
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map(User::from).collect())
    }

    async fn list_with_deleted(&self) -> AppResult<Vec<User>> {
        let models = UserEntity::find()
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map(User::from).collect())
    }

    async fn list_deleted(&self) -> AppResult<Vec<User>> {
        let models = UserEntity::find()
            .filter(user::Column::DeletedAt.is_not_null())
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map(User::from).collect())
    }
}
