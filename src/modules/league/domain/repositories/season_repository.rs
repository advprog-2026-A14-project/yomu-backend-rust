use crate::modules::league::domain::entities::clan::ClanTier;
use crate::modules::league::domain::entities::season::Season;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

/// Result type for season standings.
///
/// Each tuple contains (clan_id, clan_name, final_rank, final_score).
pub type SeasonResultRow = (Uuid, String, i64, i64);

#[async_trait]
pub trait SeasonRepository: Send + Sync {
    /// Creates a new season record.
    async fn create_season(&self, season: &Season) -> Result<(), AppError>;

    /// Retrieves the currently active season for a given tier, if any.
    async fn get_active_season(&self, tier: &ClanTier) -> Result<Option<Season>, AppError>;

    /// Retrieves a season by its ID.
    async fn get_season_by_id(&self, season_id: Uuid) -> Result<Option<Season>, AppError>;

    /// Retrieves the final standings for a completed season within a tier.
    ///
    /// Returns clans ordered by total_score DESC, with their rank and final score.
    async fn get_season_results(
        &self,
        tier: &ClanTier,
        season_id: Uuid,
    ) -> Result<Vec<SeasonResultRow>, AppError>;

    /// Marks a season as ended (sets is_active = false).
    async fn mark_season_ended(&self, season_id: Uuid) -> Result<(), AppError>;

    /// Updates a clan's tier (used for promotion/demotion).
    async fn update_clan_tier(&self, clan_id: Uuid, new_tier: &ClanTier) -> Result<(), AppError>;
}
