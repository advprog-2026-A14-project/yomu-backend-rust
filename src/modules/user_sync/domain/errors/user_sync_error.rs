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

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Sync failed: {0}")]
    SyncFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid quiz data: {0}")]
    InvalidQuizData(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl IntoResponse for UserSyncError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            UserSyncError::UserAlreadyExists(msg) => (StatusCode::CONFLICT, msg.clone()),
            UserSyncError::UserNotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            UserSyncError::SyncFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            UserSyncError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            UserSyncError::InvalidQuizData(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            UserSyncError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
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

    #[test]
    fn user_sync_error_user_not_found_maps_to_404() {
        let error = UserSyncError::UserNotFound("user_456".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn user_sync_error_invalid_quiz_data_maps_to_400() {
        let error = UserSyncError::InvalidQuizData("invalid score".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn user_sync_error_validation_error_maps_to_400() {
        let error = UserSyncError::ValidationError("missing required field".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn user_sync_error_database_error_maps_to_500() {
        let error = UserSyncError::DatabaseError("connection lost".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
