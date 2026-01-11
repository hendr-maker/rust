//! User database entity for SeaORM.

use sea_orm::entity::prelude::*;

use domain::{User, UserRole};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    /// Soft delete timestamp (NULL = active, set = deleted)
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Convert database model to domain entity
impl From<Model> for User {
    fn from(model: Model) -> Self {
        User {
            id: model.id,
            email: model.email,
            password_hash: model.password_hash,
            name: model.name,
            role: UserRole::from(model.role.as_str()),
            created_at: model.created_at,
            updated_at: model.updated_at,
            deleted_at: model.deleted_at,
        }
    }
}
