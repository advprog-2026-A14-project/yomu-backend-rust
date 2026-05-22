use std::sync::Arc;

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::AppState;
use crate::modules::gamification::application::dto::{
    CreateAchievementRequestDto, CreateAchievementResponseDto, ToggleProfileVisibilityRequestDto,
    ToggleProfileVisibilityResponseDto, UserAchievementsResponseDto,
};
use crate::modules::gamification::application::use_cases::create_achievement::CreateAchievementUseCase;
use crate::modules::gamification::application::use_cases::get_user_achievements::GetUserAchievementsUseCase;
use crate::modules::gamification::application::use_cases::toggle_achievement_profile_visibility::ToggleAchievementProfileVisibilityUseCase;
use crate::modules::gamification::infrastructure::database::postgres::PostgresAchievementRepository;
use crate::shared::domain::base_error::AppError;
use crate::shared::infrastructure::auth::claims::AuthenticatedUser;
use crate::shared::utils::response::ApiResponse;

pub async fn get_user_achievements(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<(StatusCode, Json<ApiResponse<UserAchievementsResponseDto>>), AppError> {
    let auth_user_id = Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user_id in token".into()))?;

    let include_hidden = auth_user_id == user_id;

    let achievement_repo = Arc::new(PostgresAchievementRepository::new(state.db.clone()));
    let use_case = GetUserAchievementsUseCase::new(achievement_repo);

    let data = use_case
        .execute(user_id, include_hidden)
        .await
        .map_err(AppError::InternalServer)?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success(
            "Daftar pencapaian pengguna berhasil diambil",
            data,
        )),
    ))
}

pub async fn toggle_achievement_profile_visibility(
    State(state): State<AppState>,
    Path((user_id, achievement_id)): Path<(Uuid, Uuid)>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(body): Json<ToggleProfileVisibilityRequestDto>,
) -> Result<
    (
        StatusCode,
        Json<ApiResponse<ToggleProfileVisibilityResponseDto>>,
    ),
    AppError,
> {
    let auth_user_id = Uuid::parse_str(&auth_user.user_id)
        .map_err(|_| AppError::Unauthorized("Invalid user_id in token".into()))?;

    if auth_user_id != user_id {
        return Err(AppError::Unauthorized(
            "Anda hanya dapat mengubah pencapaian profil milik sendiri.".into(),
        ));
    }

    let achievement_repo = Arc::new(PostgresAchievementRepository::new(state.db.clone()));
    let use_case = ToggleAchievementProfileVisibilityUseCase::new(achievement_repo);

    match use_case
        .execute(user_id, achievement_id, body.is_shown_on_profile)
        .await
    {
        Ok(data) => Ok((
            StatusCode::OK,
            Json(ApiResponse::success(
                "Visibilitas pencapaian di profil berhasil diperbarui",
                data,
            )),
        )),
        Err(err_msg) => {
            let status = if err_msg.contains("tidak ditemukan") {
                StatusCode::NOT_FOUND
            } else if err_msg.contains("sudah selesai") {
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

pub async fn create_achievement(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(body): Json<CreateAchievementRequestDto>,
) -> Result<(StatusCode, Json<ApiResponse<CreateAchievementResponseDto>>), AppError> {
    ensure_admin(&auth_user)?;

    let achievement_repo = Arc::new(PostgresAchievementRepository::new(state.db.clone()));
    let use_case = CreateAchievementUseCase::new(achievement_repo);

    let data = use_case.execute(body).await.map_err(|err_msg| {
        if err_msg.contains("tidak boleh")
            || err_msg.contains("harus")
            || err_msg.contains("invalid")
            || err_msg.contains("tidak valid")
        {
            AppError::BadRequest(err_msg)
        } else {
            AppError::InternalServer(err_msg)
        }
    })?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Achievement berhasil dibuat", data)),
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
