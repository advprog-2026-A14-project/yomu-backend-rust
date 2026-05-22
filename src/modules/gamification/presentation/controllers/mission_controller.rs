use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};

use crate::AppState;
use crate::modules::gamification::application::use_cases::get_daily_missions::GetDailyMissionsUseCase;
use crate::modules::gamification::infrastructure::database::postgres::PostgresMissionRepository;
use crate::shared::domain::base_error::AppError;
use crate::shared::infrastructure::auth::claims::AuthenticatedUser;
use crate::shared::utils::response::ApiResponse;

pub async fn get_daily_missions(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<(StatusCode, Json<ApiResponse<crate::modules::gamification::application::dto::DailyMissionsResponseDto>>), AppError> {
    let user_id = uuid::Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user_id in token".into()))?;

    let mission_repo = Arc::new(PostgresMissionRepository::new(state.db.clone()));
    let use_case = GetDailyMissionsUseCase::new(mission_repo);

    let data = use_case
        .execute(user_id)
        .await
        .map_err(AppError::InternalServer)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("Daftar misi harian berhasil diambil", data)),
    ))
}