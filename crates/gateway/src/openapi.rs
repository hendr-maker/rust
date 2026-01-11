//! OpenAPI documentation.

use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::clients::TokenResponse;
use crate::handlers::auth_handler::{LoginRequest, RegisterRequest};
use crate::handlers::user_handler::UpdateUserRequest;
use domain::UserResponse;

/// API documentation struct.
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth_handler::register,
        crate::handlers::auth_handler::login,
        crate::handlers::user_handler::get_current_user,
        crate::handlers::user_handler::list_users,
        crate::handlers::user_handler::get_user,
        crate::handlers::user_handler::update_user,
        crate::handlers::user_handler::delete_user,
        crate::handlers::user_handler::restore_user,
    ),
    components(
        schemas(
            RegisterRequest,
            LoginRequest,
            TokenResponse,
            UserResponse,
            UpdateUserRequest,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "User authentication endpoints"),
        (name = "Users", description = "User management endpoints"),
    )
)]
pub struct ApiDoc;

/// Security scheme modifier.
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
                        .build(),
                ),
            );
        }
    }
}
