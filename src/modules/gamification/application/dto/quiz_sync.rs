use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SyncQuizHistoryRequestDto {
    pub user_id: Uuid,
    pub article_id: Uuid,
    pub score: i32,
    pub accuracy: f64,
}
