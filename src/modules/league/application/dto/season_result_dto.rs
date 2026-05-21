use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::modules::league::domain::entities::clan::ClanTier;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ClanPromotion {
    pub clan_id: Uuid,
    pub clan_name: String,
    pub from_tier: ClanTier,
    pub to_tier: ClanTier,
    pub final_rank: i64,
    pub final_score: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ClanDemotion {
    pub clan_id: Uuid,
    pub clan_name: String,
    pub from_tier: ClanTier,
    pub to_tier: ClanTier,
    pub final_rank: i64,
    pub final_score: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SeasonResultDto {
    pub season_id: Uuid,
    pub promoted: Vec<ClanPromotion>,
    pub demoted: Vec<ClanDemotion>,
}
