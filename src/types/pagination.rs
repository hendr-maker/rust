//! Pagination types for list endpoints.

use serde::{Deserialize, Serialize};

use crate::config::{DEFAULT_PAGE_NUMBER, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};

/// Pagination query parameters (DRY - reusable across all list endpoints)
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    DEFAULT_PAGE_NUMBER
}

fn default_per_page() -> u64 {
    DEFAULT_PAGE_SIZE
}

impl PaginationParams {
    /// Calculate offset for database query
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.per_page
    }

    /// Get limit capped at maximum
    pub fn limit(&self) -> u64 {
        self.per_page.min(MAX_PAGE_SIZE)
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: DEFAULT_PAGE_NUMBER,
            per_page: DEFAULT_PAGE_SIZE,
        }
    }
}

/// Paginated response wrapper (DRY - reusable for all list responses)
#[derive(Debug, Serialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
    pub total_pages: u64,
}

impl<T> Paginated<T> {
    /// Create new paginated response
    pub fn new(data: Vec<T>, page: u64, per_page: u64, total: u64) -> Self {
        let total_pages = if per_page > 0 {
            (total + per_page - 1) / per_page
        } else {
            0
        };

        Self {
            data,
            meta: PaginationMeta {
                page,
                per_page,
                total,
                total_pages,
            },
        }
    }
}
