use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LeaderboardQueryDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
}
