use crate::shared::utils::response::ApiResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    InternalServer(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for AppError {
    /// Converts AppError into an Axum HTTP response.
    ///
    /// Maps error variants to HTTP status codes:
    /// - InternalServer -> 500
    /// - BadRequest -> 400
    /// - NotFound -> 404
    ///
    /// Logs the error using tracing::error! for observability.
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::InternalServer(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
        };

        let body = ApiResponse::<()>::error(&error_message);

        tracing::error!("Error: {:?}", self);

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_internal_server_maps_to_500() {
        let error = AppError::InternalServer("db connection lost".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn app_error_bad_request_maps_to_400() {
        let error = AppError::BadRequest("invalid input".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn app_error_not_found_maps_to_404() {
        let error = AppError::NotFound("resource not found".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn app_error_into_response_returns_correct_body() {
        let error = AppError::NotFound("item_xyz".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn app_error_logs_error_message() {
        let error = AppError::InternalServer("test error".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
