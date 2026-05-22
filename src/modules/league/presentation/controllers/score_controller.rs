use axum::{
    extract::{Path, Query, State},
    http::header::{CACHE_CONTROL, HeaderValue},
    response::{IntoResponse, Json, Response},
};

use crate::AppState;
use crate::modules::league::application::GetLeaderboardUseCase;
use crate::modules::league::application::UpdateScoreWithBuffsUseCase;
use crate::modules::league::application::dto::{LeaderboardDto, ScoreResultDto};
use crate::modules::league::infrastructure::database::postgres::ClanPostgresRepo;
use crate::modules::league::infrastructure::database::postgres::clan_buff_postgres_repo::ClanBuffPostgresRepo;
use crate::modules::league::infrastructure::database::redis::LeaderboardRedisRepo;
use crate::shared::domain::base_error::AppError;
use crate::shared::utils::response::ApiResponse;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    #[serde(default = "default_tier")]
    tier: String,
}

fn default_tier() -> String {
    "Bronze".to_string()
}

#[utoipa::path(
    get,
    path = "/api/v1/leaderboards",
    params(
        ("tier" = String, Query, description = "Leaderboard tier (Bronze, Silver, Gold, Diamond)")
    ),
    responses(
        (status = 200, description = "Leaderboard fetched successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "leaderboard"
)]
#[instrument(skip(state))]
pub async fn get_leaderboard_handler(
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Response, AppError> {
    let redis_repo = LeaderboardRedisRepo::new(state.redis);
    let clan_repo = ClanPostgresRepo::new(state.db.clone());
    let use_case = GetLeaderboardUseCase::new(clan_repo, redis_repo);

    tracing::info!(tier = %query.tier, "Fetching leaderboard");
    let leaderboard = use_case.execute(query.tier).await?;

    let mut response = Json(ApiResponse::success(
        "Leaderboard fetched successfully",
        leaderboard,
    ))
    .into_response();

    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=60"),
    );

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/clans/{id}/score",
    params(
        ("id" = Uuid, Path, description = "Clan ID")
    ),
    responses(
        (status = 200, description = "Score updated successfully"),
        (status = 404, description = "Clan not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "clans"
)]
pub async fn update_score_handler(
    State(state): State<AppState>,
    Path(clan_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ScoreResultDto>>, AppError> {
    let clan_repo = ClanPostgresRepo::new(state.db.clone());
    let buff_repo = ClanBuffPostgresRepo::new(state.db.clone());
    let redis_repo = LeaderboardRedisRepo::new(state.redis);

    let use_case = UpdateScoreWithBuffsUseCase::new(clan_repo, buff_repo, redis_repo);

    let result = use_case.execute(clan_id).await?;

    Ok(Json(ApiResponse::success(
        "Score updated successfully",
        result,
    )))
}
