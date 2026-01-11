//! OpenAPI documentation configuration.
//!
//! Provides Swagger UI for API exploration and testing.

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::api::handlers::{auth_handler, user_handler};
use crate::domain::{CreateUser, UpdateUser, UserResponse, UserRole};
use crate::services::TokenResponse;

/// OpenAPI documentation for the Rust API Starter
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rust API Starter",
        version = "0.1.0",
        description = "A production-ready Rust API with Axum, SeaORM, and clean architecture",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT"),
        contact(name = "API Support", email = "support@example.com")
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://api.example.com", description = "Production server")
    ),
    paths(
        // Authentication endpoints
        auth_handler::register,
        auth_handler::login,
        // User endpoints
        user_handler::get_current_user,
        user_handler::list_users,
        user_handler::get_user,
        user_handler::update_user,
        user_handler::delete_user,
        user_handler::restore_user,
    ),
    components(
        schemas(
            // Domain types
            UserRole,
            UserResponse,
            CreateUser,
            UpdateUser,
            // Auth types
            auth_handler::RegisterRequest,
            auth_handler::LoginRequest,
            TokenResponse,
            // User handler types
            user_handler::UpdateUserRequest,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "User registration and login"),
        (name = "Users", description = "User management operations")
    )
)]
pub struct ApiDoc;

/// Security scheme modifier for JWT Bearer authentication
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT token obtained from /auth/login"))
                        .build(),
                ),
            );
        }
    }
}
