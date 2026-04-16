use crate::shared::utils::response::ApiResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserSyncError {
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),

    #[error("Sync failed: {0}")]
    SyncFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl IntoResponse for UserSyncError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            UserSyncError::UserAlreadyExists(msg) => (StatusCode::CONFLICT, msg.clone()),
            UserSyncError::SyncFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            UserSyncError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = ApiResponse::<()>::error(&error_message);

        tracing::error!("UserSyncError: {:?}", self);

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_sync_error_user_already_exists_maps_to_409() {
        let error = UserSyncError::UserAlreadyExists("user_123".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn user_sync_error_sync_failed_maps_to_500() {
        let error = UserSyncError::SyncFailed("network timeout".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
