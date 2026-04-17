use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JoinClanDto {
    pub clan_id: Uuid,
    pub user_id: Uuid,
}
