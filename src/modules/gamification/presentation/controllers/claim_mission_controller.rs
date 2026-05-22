use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

use crate::AppState;
use crate::modules::gamification::application::use_cases::claim_mission_reward::ClaimMissionRewardUseCase;
use crate::modules::gamification::infrastructure::database::postgres::PostgresMissionRepository;
use crate::shared::domain::base_error::AppError;
use crate::shared::infrastructure::auth::claims::AuthenticatedUser;
use crate::shared::utils::response::ApiResponse;

#[instrument(skip(state))]
pub async fn claim_mission_reward(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    let user_id = Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user_id in token".into()))?;

    let mission_repo = Arc::new(PostgresMissionRepository::new(state.db.clone()));
    let use_case = ClaimMissionRewardUseCase::new(mission_repo);

    match use_case.execute(user_id, mission_id).await {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(ApiResponse::success_without_data(
                "Reward misi harian berhasil diklaim",
            )),
        )),
        Err(err_msg) => {
            let status = if err_msg.contains("tidak ditemukan") {
                StatusCode::NOT_FOUND
            } else if err_msg.contains("sudah di-claim") || err_msg.contains("belum selesai") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err(match status {
                StatusCode::NOT_FOUND => AppError::NotFound(err_msg),
                StatusCode::BAD_REQUEST => AppError::BadRequest(err_msg),
                _ => AppError::InternalServer(err_msg),
            })
        }
    }
}
