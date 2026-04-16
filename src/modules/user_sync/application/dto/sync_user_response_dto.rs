use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncUserResponseDto {
    pub user_id: Uuid,
    pub message: String,
}
