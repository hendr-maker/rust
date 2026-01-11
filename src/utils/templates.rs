//! Code generation templates.

use std::fs;
use std::path::Path;

use crate::errors::{AppError, AppResult};

/// Generate entity files
pub fn generate_entity(name: &str) -> AppResult<()> {
    let snake_name = to_snake_case(name);
    let pascal_name = to_pascal_case(name);

    // Generate domain entity
    let entity_content = format!(
        r#"//! {pascal_name} domain entity.

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use utoipa::ToSchema;
use uuid::Uuid;

/// {pascal_name} domain entity with soft delete support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal_name} {{
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Soft delete timestamp (None = active, Some = deleted)
    pub deleted_at: Option<DateTime<Utc>>,
}}

/// {pascal_name} response DTO for API responses
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct {pascal_name}Response {{
    /// Unique identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}}

impl From<{pascal_name}> for {pascal_name}Response {{
    fn from(entity: {pascal_name}) -> Self {{
        Self {{
            id: entity.id,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }}
    }}
}}
"#
    );

    write_file(&format!("src/domain/{}.rs", snake_name), &entity_content)?;

    // Generate repository
    let repo_content = format!(
        r#"//! {pascal_name} repository with soft delete support.

use async_trait::async_trait;
use sea_orm::{{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set}};
use uuid::Uuid;

use super::entities::{snake_name}::{{self, ActiveModel, Entity as {pascal_name}Entity}};
use crate::domain::{pascal_name};
use crate::errors::{{AppError, AppResult}};

#[cfg(any(test, feature = "test-utils"))]
use mockall::automock;

/// {pascal_name} repository trait for dependency injection.
///
/// By default, all query methods exclude soft-deleted records.
/// Use `*_with_deleted` variants to include them.
#[cfg_attr(any(test, feature = "test-utils"), automock)]
#[async_trait]
pub trait {pascal_name}Repository: Send + Sync {{
    /// Find active record by ID (excludes soft-deleted)
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<{pascal_name}>>;

    /// Find record by ID including soft-deleted
    async fn find_by_id_with_deleted(&self, id: Uuid) -> AppResult<Option<{pascal_name}>>;

    /// List all active records (excludes soft-deleted)
    async fn list(&self) -> AppResult<Vec<{pascal_name}>>;

    /// Create a new record
    async fn create(&self) -> AppResult<{pascal_name}>;

    /// Soft delete record by ID (sets deleted_at timestamp)
    async fn delete(&self, id: Uuid) -> AppResult<()>;

    /// Restore a soft-deleted record
    async fn restore(&self, id: Uuid) -> AppResult<{pascal_name}>;
}}

/// Concrete implementation of {pascal_name}Repository with soft delete
pub struct {pascal_name}Store {{
    db: DatabaseConnection,
}}

impl {pascal_name}Store {{
    /// Create new repository instance
    pub fn new(db: DatabaseConnection) -> Self {{
        Self {{ db }}
    }}
}}

#[async_trait]
impl {pascal_name}Repository for {pascal_name}Store {{
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<{pascal_name}>> {{
        let result = {pascal_name}Entity::find_by_id(id)
            .filter({snake_name}::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(result.map({pascal_name}::from))
    }}

    async fn find_by_id_with_deleted(&self, id: Uuid) -> AppResult<Option<{pascal_name}>> {{
        let result = {pascal_name}Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(result.map({pascal_name}::from))
    }}

    async fn list(&self) -> AppResult<Vec<{pascal_name}>> {{
        let models = {pascal_name}Entity::find()
            .filter({snake_name}::Column::DeletedAt.is_null())
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map({pascal_name}::from).collect())
    }}

    async fn create(&self) -> AppResult<{pascal_name}> {{
        let now = chrono::Utc::now();
        let active_model = ActiveModel {{
            id: Set(Uuid::new_v4()),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        }};

        let model = active_model.insert(&self.db).await.map_err(AppError::from)?;
        Ok({pascal_name}::from(model))
    }}

    async fn delete(&self, id: Uuid) -> AppResult<()> {{
        let record = {pascal_name}Entity::find_by_id(id)
            .filter({snake_name}::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut active: ActiveModel = record.into();
        let now = chrono::Utc::now();
        active.deleted_at = Set(Some(now));
        active.updated_at = Set(now);

        active.update(&self.db).await.map_err(AppError::from)?;
        Ok(())
    }}

    async fn restore(&self, id: Uuid) -> AppResult<{pascal_name}> {{
        let record = {pascal_name}Entity::find_by_id(id)
            .filter({snake_name}::Column::DeletedAt.is_not_null())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::validation("Record is not deleted or does not exist"))?;

        let mut active: ActiveModel = record.into();
        active.deleted_at = Set(None);
        active.updated_at = Set(chrono::Utc::now());

        let model = active.update(&self.db).await.map_err(AppError::from)?;
        Ok({pascal_name}::from(model))
    }}
}}
"#
    );

    write_file(
        &format!("src/infra/repositories/{}_repository.rs", snake_name),
        &repo_content,
    )?;

    Ok(())
}

/// Generate migration file
pub fn generate_migration(name: &str) -> AppResult<()> {
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let snake_name = to_snake_case(name);
    let pascal_name = to_pascal_case(name);
    let table_name = format!("{}s", snake_name); // Pluralize for table name
    let filename = format!("m{}_{}.rs", timestamp, snake_name);

    let content = format!(
        r#"//! Migration: {name}

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Table and column identifiers for {pascal_name}
#[derive(Iden)]
enum {pascal_name} {{
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}}

#[async_trait::async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .create_table(
                Table::create()
                    .table({pascal_name}::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new({pascal_name}::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new({pascal_name}::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new({pascal_name}::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new({pascal_name}::DeletedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on deleted_at for soft delete queries
        manager
            .create_index(
                Index::create()
                    .name("idx_{table_name}_deleted_at")
                    .table({pascal_name}::Table)
                    .col({pascal_name}::DeletedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .drop_table(Table::drop().table({pascal_name}::Table).to_owned())
            .await
    }}
}}
"#
    );

    write_file(&format!("migrations/{}", filename), &content)?;

    Ok(())
}

/// Generate service file
pub fn generate_service(name: &str) -> AppResult<()> {
    let snake_name = to_snake_case(name);
    let pascal_name = to_pascal_case(name);

    let content = format!(
        r#"//! {pascal_name} service - Business logic layer.
//!
//! SOLID (SRP): Handles {pascal_name} business logic only.
//! SOLID (DIP): Depends on repository abstraction, not concrete implementation.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{pascal_name};
use crate::errors::{{AppError, AppResult}};
use crate::infra::repositories::{pascal_name}Repository;

/// {pascal_name} service trait for dependency injection.
#[async_trait]
pub trait {pascal_name}Service: Send + Sync {{
    /// Get {pascal_name} by ID (excludes soft-deleted)
    async fn get(&self, id: Uuid) -> AppResult<{pascal_name}>;

    /// Get {pascal_name} by ID (includes soft-deleted)
    async fn get_with_deleted(&self, id: Uuid) -> AppResult<{pascal_name}>;

    /// List all active records (excludes soft-deleted)
    async fn list(&self) -> AppResult<Vec<{pascal_name}>>;

    /// Create a new record
    async fn create(&self) -> AppResult<{pascal_name}>;

    /// Soft delete record (sets deleted_at timestamp)
    async fn delete(&self, id: Uuid) -> AppResult<()>;

    /// Restore a soft-deleted record
    async fn restore(&self, id: Uuid) -> AppResult<{pascal_name}>;
}}

/// Concrete implementation using repository pattern.
pub struct {pascal_name}Manager {{
    repo: Arc<dyn {pascal_name}Repository>,
}}

impl {pascal_name}Manager {{
    /// Create new service instance with repository
    pub fn new(repo: Arc<dyn {pascal_name}Repository>) -> Self {{
        Self {{ repo }}
    }}
}}

#[async_trait]
impl {pascal_name}Service for {pascal_name}Manager {{
    async fn get(&self, id: Uuid) -> AppResult<{pascal_name}> {{
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)
    }}

    async fn get_with_deleted(&self, id: Uuid) -> AppResult<{pascal_name}> {{
        self.repo
            .find_by_id_with_deleted(id)
            .await?
            .ok_or(AppError::NotFound)
    }}

    async fn list(&self) -> AppResult<Vec<{pascal_name}>> {{
        self.repo.list().await
    }}

    async fn create(&self) -> AppResult<{pascal_name}> {{
        self.repo.create().await
    }}

    async fn delete(&self, id: Uuid) -> AppResult<()> {{
        self.repo.delete(id).await
    }}

    async fn restore(&self, id: Uuid) -> AppResult<{pascal_name}> {{
        self.repo.restore(id).await
    }}
}}
"#
    );

    write_file(&format!("src/services/{}_service.rs", snake_name), &content)?;

    Ok(())
}

/// Write content to file
fn write_file(path: &str, content: &str) -> AppResult<()> {
    let path = Path::new(path);

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::internal(e.to_string()))?;
    }

    fs::write(path, content).map_err(|e| AppError::internal(e.to_string()))?;

    Ok(())
}

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}
