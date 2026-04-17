use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// DTO for user tier response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserTierDto {
    pub user_id: Uuid,
    pub clan_id: Option<Uuid>,
    pub clan_name: Option<String>,
    pub tier: Option<String>,
}

impl UserTierDto {
    pub fn not_in_clan(user_id: Uuid) -> Self {
        Self {
            user_id,
            clan_id: None,
            clan_name: None,
            tier: None,
        }
    }

    pub fn from_clan(user_id: Uuid, clan_id: Uuid, clan_name: String, tier: String) -> Self {
        Self {
            user_id,
            clan_id: Some(clan_id),
            clan_name: Some(clan_name),
            tier: Some(tier),
        }
    }
}
