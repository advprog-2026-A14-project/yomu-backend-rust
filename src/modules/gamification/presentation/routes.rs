use std::sync::Arc;
use axum::{
    routing::post,
    Router,
};

use crate::modules::gamification::presentation::controllers::{
    sync_quiz_controller,
    claim_mission_controller,
};
use crate::modules::gamification::application::use_cases::{
    sync_quiz_gamification::SyncQuizGamificationUseCase,
    claim_mission_reward::ClaimMissionRewardUseCase,
};

// untuk use case yang dibutuhkan  controller
pub struct GamificationState {
    pub sync_quiz_use_case: Arc<SyncQuizGamificationUseCase>,
    pub claim_mission_use_case: Arc<ClaimMissionRewardUseCase>,
}

pub fn create_gamification_router(state: Arc<GamificationState>) -> Router {
    // internal group: Menggunakan x-api-key (validasi di controller)
    let internal_routes = Router::new()
        .route(
            "/api/internal/quiz-history/sync", 
            post(sync_quiz_controller::sync_quiz_history)
        );

    // public group: akses fitur user (nannti dikunci oleh JWT)
    let public_routes = Router::new()
        .route(
            "/api/v1/missions/:id/claim", 
            post(claim_mission_controller::claim_mission_reward)
        );
        // TODO: .route_layer(...)  untuk JWT Middleware

    // gabungkan internal group dan public group
    Router::new()
        .merge(internal_routes)
        .merge(public_routes)
        .with_state(state) // State disuntikkan ke semua rute
}