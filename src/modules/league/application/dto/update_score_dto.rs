use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateScoreDto {
    pub clan_id: Uuid,
    pub user_id: Uuid,
    pub base_score: i64,
    pub multiplier: f64,
}
