use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowUser {
    pub user_id: Uuid,
    pub total_score: i32,
}

impl ShadowUser {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            total_score: 0,
        }
    }
}