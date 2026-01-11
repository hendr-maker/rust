//! Validated JSON extractor.

use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use common::AppError;

/// JSON extractor that automatically validates the payload.
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
        // Extract JSON
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::validation(e.body_text()))?;

        // Validate
        value.validate().map_err(|e| {
            // Get first validation error message
            let message = e
                .field_errors()
                .values()
                .next()
                .and_then(|errors| errors.first())
                .and_then(|error| error.message.as_ref())
                .map(|msg| msg.to_string())
                .unwrap_or_else(|| "Validation failed".to_string());
            AppError::validation(message)
        })?;

        Ok(ValidatedJson(value))
    }
}
