//! Base repository traits following Interface Segregation Principle (ISP).
//!
//! These traits provide a foundation for all repositories with
//! common CRUD operations that can be composed as needed.

use async_trait::async_trait;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, FromQueryResult,
    IntoActiveModel, PaginatorTrait, PrimaryKeyTrait,
};
use std::fmt::Debug;

use crate::errors::AppResult;
use crate::types::PaginationParams;

/// Read operations (Query) - Single Responsibility
#[async_trait]
pub trait ReadRepository<E, M>: Send + Sync
where
    E: EntityTrait<Model = M>,
    M: Send + Sync + FromQueryResult,
{
    /// Get database connection reference
    fn db(&self) -> &DatabaseConnection;

    /// Find entity by primary key
    async fn find_by_id(&self, id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType) -> AppResult<Option<M>>
    where
        <E::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone + Send,
    {
        E::find_by_id(id)
            .one(self.db())
            .await
            .map_err(Into::into)
    }

    /// Find all entities
    async fn find_all(&self) -> AppResult<Vec<M>> {
        E::find()
            .all(self.db())
            .await
            .map_err(Into::into)
    }

    /// Find entities with pagination
    async fn find_paginated(&self, params: &PaginationParams) -> AppResult<(Vec<M>, u64)> {
        let paginator = E::find().paginate(self.db(), params.limit());
        let total = paginator.num_items().await?;
        let data = paginator.fetch_page(params.page.saturating_sub(1)).await?;
        Ok((data, total))
    }

    /// Count all entities
    async fn count(&self) -> AppResult<u64> {
        E::find()
            .paginate(self.db(), 1)
            .num_items()
            .await
            .map_err(Into::into)
    }
}

/// Write operations (Command) - Single Responsibility
#[async_trait]
pub trait WriteRepository<E, M, A>: Send + Sync
where
    E: EntityTrait<Model = M>,
    M: Send + Sync + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + 'static,
{
    /// Get database connection reference
    fn db(&self) -> &DatabaseConnection;

    /// Insert new entity
    async fn insert(&self, model: A) -> AppResult<M>
    where
        <<A as ActiveModelTrait>::Entity as EntityTrait>::Model: Send,
    {
        model
            .insert(self.db())
            .await
            .map_err(Into::into)
    }

    /// Update existing entity
    async fn update(&self, model: A) -> AppResult<M>
    where
        <<A as ActiveModelTrait>::Entity as EntityTrait>::Model: Send,
    {
        model
            .update(self.db())
            .await
            .map_err(Into::into)
    }
}

/// Delete operations - Single Responsibility
#[async_trait]
pub trait DeleteRepository<E>: Send + Sync
where
    E: EntityTrait,
{
    /// Get database connection reference
    fn db(&self) -> &DatabaseConnection;

    /// Delete entity by primary key
    async fn delete_by_id(&self, id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType) -> AppResult<()>
    where
        <E::PrimaryKey as PrimaryKeyTrait>::ValueType: Clone + Send + Debug,
    {
        E::delete_by_id(id)
            .exec(self.db())
            .await?;
        Ok(())
    }
}

/// Full CRUD repository - Combines all operations
/// Follows Open/Closed Principle: extend by implementing individual traits
pub trait CrudRepository<E, M, A>:
    ReadRepository<E, M> + WriteRepository<E, M, A> + DeleteRepository<E>
where
    E: EntityTrait<Model = M>,
    M: Send + Sync + FromQueryResult + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + 'static,
{
}

// Auto-implement CrudRepository for types implementing all traits
impl<T, E, M, A> CrudRepository<E, M, A> for T
where
    T: ReadRepository<E, M> + WriteRepository<E, M, A> + DeleteRepository<E>,
    E: EntityTrait<Model = M>,
    M: Send + Sync + FromQueryResult + IntoActiveModel<A>,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send + 'static,
{
}
