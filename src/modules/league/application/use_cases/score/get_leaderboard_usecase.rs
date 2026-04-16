use crate::modules::league::application::dto::LeaderboardDto;
use crate::modules::league::domain::repositories::LeaderboardCache;
use crate::shared::domain::base_error::AppError;

pub struct GetLeaderboardUseCase<L: LeaderboardCache> {
    leaderboard: L,
}

impl<L: LeaderboardCache> GetLeaderboardUseCase<L> {
    pub fn new(leaderboard: L) -> Self {
        Self { leaderboard }
    }

    /// Retrieves the top clans for a given leaderboard tier.
    ///
    /// Uses Redis cache for fast retrieval. Returns up to 10 clans
    /// ordered by total score descending.
    pub async fn execute(&self, tier: String) -> Result<LeaderboardDto, AppError> {
        let entries = self.leaderboard.get_top_clans(&tier, 10).await?;

        Ok(LeaderboardDto { entries, tier })
    }
}
