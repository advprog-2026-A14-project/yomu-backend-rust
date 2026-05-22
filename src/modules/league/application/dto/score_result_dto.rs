use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScoreResultDto {
    pub final_score: i64,
    pub multiplier: f64,
    pub strategy: String,
}
