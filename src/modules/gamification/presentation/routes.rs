use axum::{
    routing::{get, patch, post, delete},
    Router,
};

use crate::modules::gamification::presentation::controllers::{
    achievement_controller, claim_mission_controller, mission_controller,
};
use crate::AppState;

pub fn gamification_public_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/missions/daily",
            get(mission_controller::get_daily_missions),
        )
        .route(
            "/missions/{id}/claim",
            post(claim_mission_controller::claim_mission_reward),
        )
        .route(
            "/achievements/users/{user_id}",
            get(achievement_controller::get_user_achievements),
        )
        .route(
            "/achievements/users/{user_id}/{achievement_id}/profile-visibility",
            patch(achievement_controller::toggle_achievement_profile_visibility),
        )
        .route("/admin/achievements", post(achievement_controller::create_achievement))
        .route("/admin/missions/daily", post(mission_controller::create_daily_mission))
        .route("/admin/missions/{id}", patch(mission_controller::update_daily_mission))
        .route("/admin/missions/{id}", delete(mission_controller::delete_daily_mission))
        .route("/admin/achievements", post(achievement_controller::create_achievement))
        .route("/admin/missions/daily", post(mission_controller::create_daily_mission))
        .route("/admin/missions/{id}", patch(mission_controller::update_daily_mission))
        .route("/admin/missions/{id}", delete(mission_controller::delete_daily_mission))
}
