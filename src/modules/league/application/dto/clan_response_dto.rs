use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClanResponseDto {
    pub id: Uuid,
    pub name: String,
    pub leader_id: Uuid,
    pub tier: String,
    pub total_score: i64,
    pub member_count: i32,
}
