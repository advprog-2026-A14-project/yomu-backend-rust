use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::AppState;
use crate::modules::gamification::application::dto::UserAchievementsResponseDto;
use crate::modules::gamification::application::use_cases::get_user_achievements::GetUserAchievementsUseCase;
use crate::modules::gamification::infrastructure::database::postgres::PostgresAchievementRepository;
use crate::shared::domain::base_error::AppError;
use crate::shared::utils::response::ApiResponse;

pub async fn get_user_achievements(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<UserAchievementsResponseDto>>), AppError> {
    let achievement_repo = Arc::new(PostgresAchievementRepository::new(state.db.clone()));
    let use_case = GetUserAchievementsUseCase::new(achievement_repo);

    let data = use_case
        .execute(user_id)
        .await
        .map_err(AppError::InternalServer)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("Daftar pencapaian pengguna berhasil diambil", data)),
    ))
}