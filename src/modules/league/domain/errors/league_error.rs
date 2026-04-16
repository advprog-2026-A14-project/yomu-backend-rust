use crate::shared::domain::base_error::AppError;
use crate::shared::utils::response::ApiResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LeagueError {
    #[error("Clan not found: {0}")]
    ClanNotFound(String),

    #[error("Clan is full: {0}")]
    ClanIsFull(String),

    #[error("User already in a clan: {0}")]
    UserAlreadyInClan(String),

    #[error("User not in any clan: {0}")]
    UserNotInAnyClan(String),

    #[error("Max clans reached: {0}")]
    MaxClansReached(String),
}

impl IntoResponse for LeagueError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            LeagueError::ClanNotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            LeagueError::ClanIsFull(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            LeagueError::UserAlreadyInClan(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            LeagueError::UserNotInAnyClan(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            LeagueError::MaxClansReached(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        };

        let body = ApiResponse::<()>::error(&error_message);

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn league_error_clan_not_found_maps_to_404() {
        let error = LeagueError::ClanNotFound("clan_123".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn league_error_user_already_in_clan_maps_to_400() {
        let error = LeagueError::UserAlreadyInClan("user_456".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn league_error_into_response_body_format() {
        let error = LeagueError::ClanNotFound("clan_abc".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

impl From<AppError> for LeagueError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::InternalServer(msg) => LeagueError::ClanNotFound(msg),
            AppError::BadRequest(msg) => LeagueError::UserNotInAnyClan(msg),
            AppError::NotFound(msg) => LeagueError::ClanNotFound(msg),
        }
    }
}
