use axum::{
    extract::{Query, State},
    http::header::{CACHE_CONTROL, HeaderValue},
    response::{IntoResponse, Json, Response},
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
) -> Result<Response, AppError> {
    let redis_repo = LeaderboardRedisRepo::new(state.redis);
    let use_case = GetLeaderboardUseCase::new(redis_repo);

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
