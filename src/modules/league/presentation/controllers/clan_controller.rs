use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use tracing::instrument;

use crate::AppState;
use crate::modules::league::application::ApproveJoinRequestUseCase;
use crate::modules::league::application::CreateClanUseCase;
use crate::modules::league::application::CreateJoinRequestUseCase;
use crate::modules::league::application::DeleteClanUseCase;
use crate::modules::league::application::GetClanDetailUseCase;
use crate::modules::league::application::GetPendingRequestsUseCase;
use crate::modules::league::application::JoinClanUseCase;
use crate::modules::league::application::ProcessBuffsUseCase;
use crate::modules::league::application::RejectJoinRequestUseCase;
use crate::modules::league::application::dto::{
    ApproveRejectDto, BuffProcessResultDto, ClanDetailDto, CreateClanDto, CreateJoinRequestDto,
    DeleteClanDto, JoinClanDto, JoinRequestResponseDto, SeasonResultDto, user_tier_dto::UserTierDto,
};
use crate::modules::league::application::use_cases::GetUserTierUseCase;
use crate::modules::league::application::use_cases::TriggerSeasonEndUseCase;
use crate::modules::league::domain::entities::clan::Clan;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::modules::league::domain::errors::LeagueError;
use crate::modules::league::infrastructure::database::postgres::clan_buff_postgres_repo::ClanBuffPostgresRepo;
use crate::modules::league::infrastructure::database::postgres::clan_join_request_postgres_repo::ClanJoinRequestPostgresRepo;
use crate::modules::league::infrastructure::database::postgres::ClanPostgresRepo;
use crate::modules::league::infrastructure::database::postgres::SeasonPostgresRepo;
use crate::modules::league::infrastructure::database::redis::LeaderboardRedisRepo;
use crate::shared::domain::base_error::AppError;
use crate::shared::utils::response::ApiResponse;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/v1/clans",
    request_body = CreateClanDto,
    responses(
        (status = 201, description = "Clan created successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "clans"
)]
#[instrument(skip(state))]
pub async fn create_clan_handler(
    State(state): State<AppState>,
    Json(dto): Json<CreateClanDto>,
) -> Result<(StatusCode, Json<ApiResponse<Clan>>), AppError> {
    let repo = ClanPostgresRepo::new(state.db);
    let use_case = CreateClanUseCase::new(repo);

    let clan = use_case.execute(dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Clan created successfully", clan)),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/clans/{id}/join",
    params(
        ("id" = Uuid, Path, description = "Clan ID")
    ),
    request_body = JoinClanDto,
    responses(
        (status = 200, description = "Joined clan successfully"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Clan not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "clans"
)]
#[instrument(skip(state))]
pub async fn join_clan_handler(
    State(state): State<AppState>,
    Path(_clan_id): Path<uuid::Uuid>,
    Json(dto): Json<JoinClanDto>,
) -> Result<Json<ApiResponse<ClanMember>>, AppError> {
    let repo = ClanPostgresRepo::new(state.db);
    let use_case = JoinClanUseCase::new(repo);

    let member = use_case.execute(dto).await?;

    Ok(Json(ApiResponse::success(
        "Joined clan successfully",
        member,
    )))
}

/// GET /api/v1/clans/{id}
/// Returns detailed clan information including members and active buffs/debuffs
#[utoipa::path(
    get,
    path = "/api/v1/clans/{id}",
    params(
        ("id" = Uuid, Path, description = "Clan ID")
    ),
    responses(
        (status = 200, description = "Clan detail retrieved"),
        (status = 404, description = "Clan not found")
    ),
    tag = "League"
)]
#[instrument(skip(state))]
pub async fn get_clan_detail_handler(
    State(state): State<AppState>,
    Path(clan_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ClanDetailDto>>, LeagueError> {
    let repository = ClanPostgresRepo::new(state.db.clone());
    let buff_repository = ClanBuffPostgresRepo::new(state.db.clone());
    let use_case = GetClanDetailUseCase::new(repository, buff_repository);

    let clan_detail = use_case.execute(clan_id).await?;

    Ok(Json(ApiResponse::success(
        "Clan detail retrieved",
        clan_detail,
    )))
}

/// GET /api/v1/league/users/{user_id}/tier
/// Returns user's clan tier information
#[utoipa::path(
    get,
    path = "/api/v1/league/users/{user_id}/tier",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User tier info retrieved"),
        (status = 400, description = "Invalid user ID")
    ),
    tag = "League"
)]
#[instrument(skip(state))]
pub async fn get_user_tier_handler(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserTierDto>>, LeagueError> {
    let repository = ClanPostgresRepo::new(state.db.clone());
    let use_case = GetUserTierUseCase::new(repository);

    let tier_info = use_case.execute(user_id).await?;

    Ok(Json(ApiResponse::success(
        "Data liga pengguna berhasil diambil",
        tier_info,
    )))
}

/// DELETE /api/v1/clans/{id}
/// Deletes a clan. Only the clan leader can perform this action.
#[utoipa::path(
    delete,
    path = "/api/v1/clans/{id}",
    params(
        ("id" = Uuid, Path, description = "Clan ID")
    ),
    request_body = DeleteClanDto,
    responses(
        (status = 200, description = "Clan deleted successfully"),
        (status = 403, description = "Only the clan leader can delete"),
        (status = 404, description = "Clan not found")
    ),
    tag = "clans"
)]
#[instrument(skip(state))]
pub async fn delete_clan_handler(
    State(state): State<AppState>,
    Path(clan_id): Path<Uuid>,
    Json(dto): Json<DeleteClanDto>,
) -> Result<Json<ApiResponse<()>>, LeagueError> {
    let repo = ClanPostgresRepo::new(state.db.clone());
    let redis_repo = LeaderboardRedisRepo::new(state.redis);
    let use_case = DeleteClanUseCase::new(repo, redis_repo);

    use_case.execute(clan_id, dto).await?;

    Ok(Json(ApiResponse::success("Clan deleted successfully", ())))
}

/// POST /api/v1/clans/{id}/join-request
/// Creates a pending join request. Clan leader must approve.
#[utoipa::path(
    post,
    path = "/api/v1/clans/{id}/join-request",
    params(
        ("id" = Uuid, Path, description = "Clan ID")
    ),
    request_body = CreateJoinRequestDto,
    responses(
        (status = 201, description = "Join request created"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Clan not found"),
        (status = 409, description = "Duplicate request")
    ),
    tag = "clans"
)]
#[instrument(skip(state))]
pub async fn create_join_request_handler(
    State(state): State<AppState>,
    Path(clan_id): Path<Uuid>,
    Json(dto): Json<CreateJoinRequestDto>,
) -> Result<(StatusCode, Json<ApiResponse<JoinRequestResponseDto>>), LeagueError> {
    let clan_repo = ClanPostgresRepo::new(state.db.clone());
    let join_repo = ClanJoinRequestPostgresRepo::new(state.db);
    let use_case = CreateJoinRequestUseCase::new(clan_repo, join_repo);

    let result = use_case.execute(dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Join request created", result)),
    ))
}

/// GET /api/v1/clans/{id}/join-requests
/// Get pending join requests (leader only)
#[utoipa::path(
    get,
    path = "/api/v1/clans/{id}/join-requests",
    params(
        ("id" = Uuid, Path, description = "Clan ID"),
        ("caller_id" = Uuid, Query, description = "Caller user ID for auth")
    ),
    responses(
        (status = 200, description = "Pending requests retrieved"),
        (status = 403, description = "Only leader can view"),
        (status = 404, description = "Clan not found")
    ),
    tag = "clans"
)]
#[instrument(skip(state))]
pub async fn get_pending_requests_handler(
    State(state): State<AppState>,
    Path(clan_id): Path<Uuid>,
    Query(dto): Query<ApproveRejectDto>,
) -> Result<Json<ApiResponse<Vec<JoinRequestResponseDto>>>, LeagueError> {
    let clan_repo = ClanPostgresRepo::new(state.db.clone());
    let join_repo = ClanJoinRequestPostgresRepo::new(state.db);
    let use_case = GetPendingRequestsUseCase::new(clan_repo, join_repo);

    let requests = use_case.execute(clan_id, dto.caller_id).await?;

    Ok(Json(ApiResponse::success("Pending requests retrieved", requests)))
}

/// POST /api/v1/clans/join-requests/{request_id}/approve
/// Approve a pending join request (leader only)
#[utoipa::path(
    post,
    path = "/api/v1/clans/join-requests/{request_id}/approve",
    params(
        ("request_id" = Uuid, Path, description = "Join request ID")
    ),
    request_body = ApproveRejectDto,
    responses(
        (status = 200, description = "Request approved"),
        (status = 400, description = "User already in clan"),
        (status = 403, description = "Only leader can approve"),
        (status = 404, description = "Request not found"),
        (status = 409, description = "Request already processed")
    ),
    tag = "clans"
)]
#[instrument(skip(state))]
pub async fn approve_join_request_handler(
    State(state): State<AppState>,
    Path(request_id): Path<Uuid>,
    Json(dto): Json<ApproveRejectDto>,
) -> Result<Json<ApiResponse<JoinRequestResponseDto>>, LeagueError> {
    let clan_repo = ClanPostgresRepo::new(state.db.clone());
    let join_repo = ClanJoinRequestPostgresRepo::new(state.db);
    let use_case = ApproveJoinRequestUseCase::new(clan_repo, join_repo);

    let result = use_case.execute(request_id, dto).await?;

    Ok(Json(ApiResponse::success("Join request approved", result)))
}

/// POST /api/v1/clans/join-requests/{request_id}/reject
/// Reject a pending join request (leader only)
#[utoipa::path(
    post,
    path = "/api/v1/clans/join-requests/{request_id}/reject",
    params(
        ("request_id" = Uuid, Path, description = "Join request ID")
    ),
    request_body = ApproveRejectDto,
    responses(
        (status = 200, description = "Request rejected"),
        (status = 403, description = "Only leader can reject"),
        (status = 404, description = "Request not found"),
        (status = 409, description = "Request already processed")
    ),
    tag = "clans"
)]
#[instrument(skip(state))]
pub async fn reject_join_request_handler(
    State(state): State<AppState>,
    Path(request_id): Path<Uuid>,
    Json(dto): Json<ApproveRejectDto>,
) -> Result<Json<ApiResponse<JoinRequestResponseDto>>, LeagueError> {
    let clan_repo = ClanPostgresRepo::new(state.db.clone());
    let join_repo = ClanJoinRequestPostgresRepo::new(state.db);
    let use_case = RejectJoinRequestUseCase::new(clan_repo, join_repo);

    let result = use_case.execute(request_id, dto).await?;

    Ok(Json(ApiResponse::success("Join request rejected", result)))
}

#[utoipa::path(
    post,
    path = "/api/v1/clans/{id}/process-buffs",
    params(
        ("id" = Uuid, Path, description = "Clan ID")
    ),
    responses(
        (status = 200, description = "Buffs processed"),
        (status = 404, description = "Clan not found")
    ),
    tag = "clans"
)]
#[instrument(skip(state))]
pub async fn process_buffs_handler(
    State(state): State<AppState>,
    Path(clan_id): Path<Uuid>,
) -> Result<Json<ApiResponse<BuffProcessResultDto>>, AppError> {
    let clan_repo = ClanPostgresRepo::new(state.db.clone());
    let buff_repo = ClanBuffPostgresRepo::new(state.db.clone());
    let use_case = ProcessBuffsUseCase::new(clan_repo, buff_repo);

    let result = use_case.execute(clan_id).await?;

    Ok(Json(ApiResponse::success("Buffs processed", result)))
}

#[utoipa::path(
    post,
    path = "/api/v1/seasons/{id}/end",
    params(
        ("id" = Uuid, Path, description = "Season ID")
    ),
    responses(
        (status = 200, description = "Season ended and clans promoted/demoted"),
        (status = 404, description = "Season not found")
    ),
    tag = "league"
)]
#[instrument(skip(state))]
pub async fn trigger_season_end_handler(
    State(state): State<AppState>,
    Path(season_id): Path<Uuid>,
) -> Result<Json<ApiResponse<SeasonResultDto>>, AppError> {
    let repo = SeasonPostgresRepo::new(state.db);
    let use_case = TriggerSeasonEndUseCase::new(repo);

    let result = use_case.execute(season_id).await?;

    Ok(Json(ApiResponse::success("Season ended", result)))
}
