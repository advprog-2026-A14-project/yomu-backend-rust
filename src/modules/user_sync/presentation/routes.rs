use super::controllers::internal_user_controller;
use crate::AppState;
use axum::{Router, routing::post};

pub fn user_sync_routes() -> Router<AppState> {
    Router::new().route("/sync", post(internal_user_controller::sync_user_handler))
}
