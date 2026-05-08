use serde::{Deserialize, Serialize};

/// JWT claims structure shared between Java Core and Rust Engine.
///
/// Java generates tokens with these claims. Rust validates them
/// independently using the shared JWT_SECRET.
///
/// # Fields
/// - `sub`: User ID (UUID v4)
/// - `role`: User role — `"PELAJAR"` or `"ADMIN"`
/// - `iat`: Issued at (Unix timestamp)
/// - `exp`: Expiration (Unix timestamp)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub role: String,
}
