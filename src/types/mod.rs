//! Shared types for DRY compliance.

mod pagination;
mod response;

pub use pagination::{Paginated, PaginationMeta, PaginationParams};
pub use response::{ApiResponse, Created, MessageResponse, NoContent};
