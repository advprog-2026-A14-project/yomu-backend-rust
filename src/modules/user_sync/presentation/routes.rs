use super::controllers::{internal_user_controller, quiz_history_controller};
use crate::AppState;
use axum::{Router, routing::post};

pub fn user_sync_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/users/sync",
            post(internal_user_controller::sync_user_handler),
        )
        .route(
            "/quiz-history/sync",
            post(quiz_history_controller::sync_quiz_history_handler),
        )
}
