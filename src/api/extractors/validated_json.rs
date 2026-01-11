//! Validated JSON extractor - Combines deserialization with validation.

use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::errors::AppError;

/// Validated JSON extractor that automatically validates requests.
///
/// # Example
///
/// ```rust,ignore
/// use serde::Deserialize;
/// use validator::Validate;
/// use rust_api_starter::api::extractors::ValidatedJson;
///
/// #[derive(Deserialize, Validate)]
/// struct CreateUserRequest {
///     #[validate(email)]
///     email: String,
///     #[validate(length(min = 8))]
///     password: String,
/// }
///
/// async fn create_user(ValidatedJson(payload): ValidatedJson<CreateUserRequest>) {
///     // payload is already validated
/// }
/// ```
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::validation(e.body_text()))?;

        value
            .validate()
            .map_err(|e| AppError::validation(format_validation_errors(&e)))?;

        Ok(ValidatedJson(value))
    }
}

/// Format validation errors into a user-friendly string
fn format_validation_errors(errors: &validator::ValidationErrors) -> String {
    errors
        .field_errors()
        .iter()
        .flat_map(|(field, errs)| {
            errs.iter().map(move |e| {
                e.message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("{} is invalid", field))
            })
        })
        .collect::<Vec<_>>()
        .join(", ")
}
