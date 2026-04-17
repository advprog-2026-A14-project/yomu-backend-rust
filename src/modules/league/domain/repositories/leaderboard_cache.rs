use crate::modules::league::application::dto::LeaderboardEntry;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait LeaderboardCache: Send + Sync {
    async fn update_clan_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
    async fn get_top_clans(
        &self,
        tier: &str,
        limit: usize,
    ) -> Result<Vec<LeaderboardEntry>, AppError>;
}
