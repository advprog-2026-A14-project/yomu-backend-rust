use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClanBuffsDto {
    pub buffs: Vec<BuffDto>,
    pub debuffs: Vec<DebuffDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BuffDto {
    pub name: String,
    pub multiplier: f64,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DebuffDto {
    pub name: String,
    pub multiplier: f64,
    pub expires_at: String,
}
