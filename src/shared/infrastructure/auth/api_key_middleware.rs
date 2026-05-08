use axum::extract::State;
use axum::{extract::Request, middleware::Next, response::Response};

use crate::AppState;
use crate::shared::domain::base_error::AppError;

/// Tower-layer compatible API key validation middleware for internal endpoints.
///
/// Validates the `x-api-key` header against `JAVA_CORE_API_KEY` env var.
/// Used to protect `/api/internal/*` routes from unauthorized access.
///
/// # Errors
/// Returns `AppError::Unauthorized` for:
/// - Missing `x-api-key` header
/// - Mismatched API key
pub async fn api_key_auth_layer(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let api_key = request
        .headers()
        .get("x-api-key")
        .and_then(|h| h.to_str().ok());

    match api_key {
        Some(key) if key == state.java_core_api_key && !state.java_core_api_key.is_empty() => {
            Ok(next.run(request).await)
        }
        _ => {
            tracing::warn!("Internal API request rejected: invalid or missing x-api-key");
            Err(AppError::Unauthorized(
                "Invalid or missing x-api-key header".to_string(),
            ))
        }
    }
}
