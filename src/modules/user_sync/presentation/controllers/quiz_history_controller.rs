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
        .execute(dto)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

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
