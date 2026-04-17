use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClanDetailDto {
    pub id: Uuid,
    pub name: String,
    pub leader_id: Uuid,
    pub tier: String,
    pub total_score: i64,
    pub created_at: DateTime<Utc>,
    pub members: Vec<ClanMemberDto>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub active_buffs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub active_debuffs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClanMemberDto {
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}
