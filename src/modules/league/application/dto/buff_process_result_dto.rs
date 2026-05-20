use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BuffProcessResultDto {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub buffs_added: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub buffs_removed: Vec<String>,
}
