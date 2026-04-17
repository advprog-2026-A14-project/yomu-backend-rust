use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuizHistoryRequestDto {
    pub user_id: Uuid,
    pub article_id: Uuid,
    pub score: i32,
    pub accuracy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuizHistoryResponseDto {
    pub user_id: Uuid,
    pub missions_updated: i32,
    pub message: String,
}
