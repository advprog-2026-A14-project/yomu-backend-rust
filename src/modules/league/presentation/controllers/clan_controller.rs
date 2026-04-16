use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

use crate::AppState;
use crate::modules::league::application::CreateClanUseCase;
use crate::modules::league::application::JoinClanUseCase;
use crate::modules::league::application::dto::{CreateClanDto, JoinClanDto};
use crate::modules::league::domain::entities::clan::Clan;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::modules::league::infrastructure::database::postgres::ClanPostgresRepo;
use crate::shared::domain::base_error::AppError;
use crate::shared::utils::response::ApiResponse;

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
