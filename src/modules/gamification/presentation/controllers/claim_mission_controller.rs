use std::sync::Arc;
use axum::{
    extract::{State, Path, Extension},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use tracing::instrument;

use crate::shared::utils::response::ApiResponse;
use crate::modules::gamification::presentation::routes::GamificationState;

#[instrument(skip(state))]
pub async fn claim_mission_reward(
    State(state): State<Arc<GamificationState>>,
    Path(mission_id): Path<Uuid>,
    Extension(user_id): Extension<Uuid>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    
    match state.claim_mission_use_case.execute(user_id, mission_id).await {
        Ok(_) => {
            (
                StatusCode::OK, // 200
                Json(ApiResponse {
                    success: true,
                    message: "Reward misi harian berhasil diklaim".to_string(),
                    data: None,
                })
            )
        },
        Err(err_msg) => {
            let status = if err_msg.contains("tidak ditemukan") {
                StatusCode::NOT_FOUND // 404
            } else if err_msg.contains("sudah di-claim") || err_msg.contains("belum selesai") {
                StatusCode::BAD_REQUEST // 400
            } else {
                StatusCode::INTERNAL_SERVER_ERROR // 500
            };

            (
                status,
                Json(ApiResponse {
                    success: false,
                    message: err_msg,
                    data: None,
                })
            )
        }
    }
}