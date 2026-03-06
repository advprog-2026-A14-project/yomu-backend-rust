use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

use crate::AppState;
use crate::modules::league::application::CreateClanUseCase;
use crate::modules::league::application::JoinClanUseCase;
use crate::modules::league::application::dto::{CreateClanDto, JoinClanDto};
use crate::modules::league::infrastructure::database::postgres::ClanPostgresRepo;
use crate::shared::domain::base_error::AppError;
use crate::shared::utils::response::ApiResponse;

pub async fn create_clan_handler(
    State(state): State<AppState>,
    Json(dto): Json<CreateClanDto>,
) -> Result<
    (
        StatusCode,
        Json<ApiResponse<crate::modules::league::domain::entities::clan::Clan>>,
    ),
    AppError,
> {
    let repo = ClanPostgresRepo::new(state.db);
    let use_case = CreateClanUseCase::new(repo);

    let clan = use_case.execute(dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Clan created successfully", clan)),
    ))
}

pub async fn join_clan_handler(
    State(state): State<AppState>,
    Path(_clan_id): Path<uuid::Uuid>,
    Json(dto): Json<JoinClanDto>,
) -> Result<
    Json<ApiResponse<crate::modules::league::domain::entities::clan_member::ClanMember>>,
    AppError,
> {
    let repo = ClanPostgresRepo::new(state.db);
    let use_case = JoinClanUseCase::new(repo);

    let member = use_case.execute(dto).await?;

    Ok(Json(ApiResponse::success(
        "Joined clan successfully",
        member,
    )))
}
