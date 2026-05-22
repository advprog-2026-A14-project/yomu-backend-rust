use std::sync::Arc;
use axum::{
    routing::{post, get},
    Router,
};

use crate::modules::gamification::presentation::controllers::{
    claim_mission_controller, achievement_controller, mission_controller
};

use crate::AppState;
// untuk use case yang dibutuhkan  controller
pub fn gamification_public_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/missions/:id/claim",
            post(claim_mission_controller::claim_mission_reward),
        )

        .route(
            "/missions/daily",
            get(mission_controller::get_daily_missions),
        )

        .route(
            "/achievements/users/:user_id",
            get(achievement_controller::get_user_achievements),
        )
}