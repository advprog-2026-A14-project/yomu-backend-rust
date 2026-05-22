use std::sync::Arc;
use axum::{
    routing::post,
    Router,
};

use crate::modules::gamification::presentation::controllers::{
    claim_mission_controller,
};
use crate::modules::gamification::application::use_cases::{
    sync_quiz_gamification::SyncQuizGamificationUseCase,
    claim_mission_reward::ClaimMissionRewardUseCase,
};
use crate::AppState;
// untuk use case yang dibutuhkan  controller
pub struct GamificationState {
    pub sync_quiz_use_case: Arc<SyncQuizGamificationUseCase>,
    pub claim_mission_use_case: Arc<ClaimMissionRewardUseCase>,
}

pub fn gamification_public_routes() -> Router<AppState> {
    Router::new().route(
        "/missions/:id/claim",
        post(claim_mission_controller::claim_mission_reward),
    )
}