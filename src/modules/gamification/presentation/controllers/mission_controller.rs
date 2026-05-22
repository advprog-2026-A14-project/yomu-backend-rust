use std::sync::Arc;

use axum::{
    extract::{Extension, State, Path},
    http::StatusCode,
    Json,
};

use crate::AppState;
use crate::modules::gamification::application::use_cases::{
    CreateDailyMissionUseCase, DeleteDailyMissionUseCase, GetDailyMissionsUseCase,
    UpdateDailyMissionUseCase,
};
use crate::modules::gamification::infrastructure::database::postgres::PostgresMissionRepository;
use crate::shared::domain::base_error::AppError;
use crate::shared::infrastructure::auth::claims::AuthenticatedUser;
use crate::shared::utils::response::ApiResponse;
use crate::modules::gamification::application::dto::{
    DailyMissionAdminRequestDto, DailyMissionItemDto, DailyMissionsResponseDto,
};

use uuid::Uuid;

pub async fn get_daily_missions(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<(StatusCode, Json<ApiResponse<DailyMissionsResponseDto>>), AppError> {
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

pub async fn create_daily_mission(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(body): Json<DailyMissionAdminRequestDto>,
) -> Result<(StatusCode, Json<ApiResponse<DailyMissionItemDto>>), AppError> {
    ensure_admin(&auth_user)?;

    let mission_repo = Arc::new(PostgresMissionRepository::new(state.db.clone()));
    let use_case = CreateDailyMissionUseCase::new(mission_repo);

    let data = use_case
        .execute(body)
        .await
        .map_err(map_mission_error)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Misi harian berhasil dibuat", data)),
    ))
}

pub async fn update_daily_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(body): Json<DailyMissionAdminRequestDto>,
) -> Result<(StatusCode, Json<ApiResponse<DailyMissionItemDto>>), AppError> {
    ensure_admin(&auth_user)?;

    let mission_repo = Arc::new(PostgresMissionRepository::new(state.db.clone()));
    let use_case = UpdateDailyMissionUseCase::new(mission_repo);

    let data = use_case
        .execute(mission_id, body)
        .await
        .map_err(map_mission_error)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("Misi harian berhasil diperbarui", data)),
    ))
}

pub async fn delete_daily_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    ensure_admin(&auth_user)?;

    let mission_repo = Arc::new(PostgresMissionRepository::new(state.db.clone()));
    let use_case = DeleteDailyMissionUseCase::new(mission_repo);

    use_case
        .execute(mission_id)
        .await
        .map_err(map_mission_error)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success_without_data(
            "Misi harian berhasil dihapus",
        )),
    ))
}

fn ensure_admin(auth_user: &AuthenticatedUser) -> Result<(), AppError> {
    if auth_user.role != "ADMIN" {
        return Err(AppError::Unauthorized(
            "Hanya admin yang dapat mengakses endpoint ini.".into(),
        ));
    }

    Ok(())
}

fn map_mission_error(err_msg: String) -> AppError {
    if err_msg.contains("tidak ditemukan") {
        AppError::NotFound(err_msg)
    } else if err_msg.contains("tidak valid")
        || err_msg.contains("tidak boleh")
        || err_msg.contains("harus")
    {
        AppError::BadRequest(err_msg)
    } else {
        AppError::InternalServer(err_msg)
    }
}