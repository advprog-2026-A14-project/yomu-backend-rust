use std::sync::Arc;

use crate::{
    AppState,
    modules::user_sync::{
        application::dto::QuizHistoryRequestDto,
        application::use_cases::sync_quiz_history_usecase::SyncQuizHistoryUseCase,
        infrastructure::database::postgres::quiz_history_postgres_repo::QuizHistoryPostgresRepo,
        infrastructure::database::postgres::user_postgres_repo::UserPostgresRepo,
    },
    shared::domain::base_error::AppError,
    shared::utils::response::ApiResponse,
};
use axum::{Json, extract::State, http::StatusCode};
use utoipa::ToSchema;

use crate::modules::gamification::application::dto::quiz_sync::SyncQuizHistoryRequestDto;
use crate::modules::gamification::application::use_cases::sync_quiz_gamification::SyncQuizGamificationUseCase;
use crate::modules::gamification::infrastructure::database::postgres::{
    PostgresAchievementRepository, PostgresMissionRepository,
};
use crate::modules::user_sync::domain::errors::UserSyncError;

#[derive(serde::Serialize, ToSchema)]
pub struct QuizHistoryApiResponse {
    pub user_id: uuid::Uuid,
    pub missions_updated: i32,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/api/internal/quiz-history/sync",
    request_body = QuizHistoryRequestDto,
    responses(
        (status = 201, description = "Quiz history synced successfully"),
        (status = 400, description = "Invalid quiz data (negative score or invalid accuracy)"),
        (status = 404, description = "User not found in Engine DB"),
        (status = 500, description = "Internal server error")
    ),
    tag = "User Sync"
)]
pub async fn sync_quiz_history_handler(
    State(state): State<AppState>,
    Json(dto): Json<QuizHistoryRequestDto>,
) -> Result<(StatusCode, Json<ApiResponse<QuizHistoryApiResponse>>), AppError> {
    let user_repo = UserPostgresRepo::new(state.db.clone());
    let quiz_repo = QuizHistoryPostgresRepo::new(state.db.clone());
    let use_case = SyncQuizHistoryUseCase::new(user_repo, quiz_repo);

    let response = use_case
        .execute(dto.clone())
        .await
        .map_err(|e| match e {
            UserSyncError::InvalidQuizData(msg) => AppError::BadRequest(msg),
            UserSyncError::UserNotFound(msg) => AppError::NotFound(msg),
            other => AppError::InternalServer(other.to_string()),
    })?;

    // Option A: trigger gamification (missions + achievements) after quiz is saved.
    // Fault tolerance §7.2: gamification failure must NOT cause quiz sync to fail.
    let gamification_payload = SyncQuizHistoryRequestDto {
        user_id: dto.user_id,
        article_id: dto.article_id,
        score: dto.score,
        accuracy: dto.accuracy,
    };

    let mission_repo = Arc::new(PostgresMissionRepository::new(state.db.clone()));
    let achievement_repo = Arc::new(PostgresAchievementRepository::new(state.db.clone()));
    let gamification_uc = SyncQuizGamificationUseCase::new(mission_repo, achievement_repo);

    if let Err(e) = gamification_uc.execute(gamification_payload).await {
        tracing::warn!(
            user_id = %dto.user_id,
            error = %e,
            "Gamification sync failed after quiz history saved — quiz result preserved"
        );
    }

    let api_response = QuizHistoryApiResponse {
        user_id: response.user_id,
        missions_updated: response.missions_updated,
        message: response.message,
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Data riwayat kuis berhasil dicatat dan diproses oleh Engine",
            api_response,
        )),
    ))
}