use axum::extract::State;
use axum::{extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use crate::AppState;
use crate::shared::domain::base_error::AppError;
use crate::shared::infrastructure::auth::claims::{AuthenticatedUser, Claims};

/// Tower-layer compatible JWT authentication middleware.
///
/// Extracts the `Authorization: Bearer <token>` header, validates the JWT
/// using the shared `JWT_SECRET`, and injects `AuthenticatedUser` into
/// request extensions for downstream handlers.
///
/// # Errors
/// Returns `AppError::Unauthorized` for:
/// - Missing Authorization header
/// - Malformed Bearer token
/// - Invalid or expired JWT
/// - Missing required claims (`sub`, `role`)
pub async fn jwt_auth_layer(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return Err(AppError::Unauthorized(
                "Missing or malformed Authorization header".to_string(),
            ));
        }
    };

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["exp", "sub"]);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|e| {
        tracing::warn!(error = %e, "JWT validation failed");
        AppError::Unauthorized(format!("Invalid token: {}", e))
    })?;

    let claims = token_data.claims;

    if claims.sub.is_empty() || claims.role.is_empty() {
        return Err(AppError::Unauthorized(
            "Invalid token claims: missing sub or role".to_string(),
        ));
    }

    let user = AuthenticatedUser {
        user_id: claims.sub,
        role: claims.role,
    };

    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}
