use axum::{
    extract::{Query, State},
    response::Json,
};

use crate::AppState;
use crate::modules::league::application::GetLeaderboardUseCase;
use crate::modules::league::application::dto::LeaderboardDto;
use crate::modules::league::infrastructure::database::redis::LeaderboardRedisRepo;
use crate::shared::domain::base_error::AppError;
use crate::shared::utils::response::ApiResponse;
use serde::Deserialize;

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
pub async fn get_leaderboard_handler(
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<ApiResponse<LeaderboardDto>>, AppError> {
    let redis_repo = LeaderboardRedisRepo::new(state.redis);
    let use_case = GetLeaderboardUseCase::new(redis_repo);

    let leaderboard = use_case.execute(query.tier).await?;

    Ok(Json(ApiResponse::success(
        "Leaderboard fetched successfully",
        leaderboard,
    )))
}
