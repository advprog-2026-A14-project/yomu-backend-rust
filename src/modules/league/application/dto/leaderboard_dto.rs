use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LeaderboardEntry {
    pub clan_id: Uuid,
    pub clan_name: String,
    pub total_score: i64,
    pub tier: String,
    pub rank: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LeaderboardDto {
    pub entries: Vec<LeaderboardEntry>,
    pub tier: String,
}
