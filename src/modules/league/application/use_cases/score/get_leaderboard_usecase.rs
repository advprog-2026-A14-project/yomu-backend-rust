use crate::modules::league::application::dto::LeaderboardDto;
use crate::modules::league::domain::repositories::{ClanRepository, LeaderboardCache};
use crate::shared::domain::base_error::AppError;

pub struct GetLeaderboardUseCase<R: ClanRepository, L: LeaderboardCache> {
    clan_repo: R,
    leaderboard: L,
}

impl<R: ClanRepository, L: LeaderboardCache> GetLeaderboardUseCase<R, L> {
    pub fn new(clan_repo: R, leaderboard: L) -> Self {
        Self {
            clan_repo,
            leaderboard,
        }
    }

    pub async fn execute(&self, tier: String) -> Result<LeaderboardDto, AppError> {
        let entries = self.leaderboard.get_top_clans(&tier, 10).await?;

        let clan_ids: Vec<uuid::Uuid> = entries.iter().map(|e| e.clan_id).collect();
        let leader_map = self.clan_repo.get_leaders_by_clan_ids(&clan_ids).await?;
        let name_map = self.clan_repo.get_clan_names_by_ids(&clan_ids).await?;

        let entries_with_data = entries
            .into_iter()
            .map(|mut e| {
                e.leader_id = leader_map
                    .get(&e.clan_id)
                    .copied()
                    .unwrap_or(uuid::Uuid::nil());
                if let Some(name) = name_map.get(&e.clan_id) {
                    e.clan_name = name.clone();
                }
                e
            })
            .collect();

        Ok(LeaderboardDto {
            entries: entries_with_data,
            tier,
        })
    }
}
