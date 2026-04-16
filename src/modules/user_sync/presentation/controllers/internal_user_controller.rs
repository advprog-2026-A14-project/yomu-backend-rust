use crate::{
    AppState,
    modules::user_sync::{
        application::dto::SyncUserRequestDto,
        application::use_cases::sync_new_user_usecase::SyncNewUserUseCase,
        infrastructure::database::postgres::user_postgres_repo::UserPostgresRepo,
    },
    shared::domain::base_error::AppError,
    shared::utils::response::ApiResponse,
};
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SyncUserResponseDto {
    pub user_id: uuid::Uuid,
    pub message: String,
}

pub async fn sync_user_handler(
    State(state): State<AppState>,
    Json(dto): Json<SyncUserRequestDto>,
) -> Result<(StatusCode, Json<ApiResponse<SyncUserResponseDto>>), AppError> {
    let repository = UserPostgresRepo::new(state.db.clone());
    let use_case = SyncNewUserUseCase::new(repository);
    let user_id = dto.user_id;

    let _ = use_case
        .execute(dto)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

    let response = SyncUserResponseDto {
        user_id,
        message: "Shadow user berhasil disinkronisasi".to_string(),
    };

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "Shadow user berhasil disinkronisasi",
            response,
        )),
    ))
}
