use crate::modules::league::application::dto::DeleteClanDto;
use crate::modules::league::domain::errors::LeagueError;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::modules::league::domain::repositories::LeaderboardCache;
use tracing::instrument;
use uuid::Uuid;

pub struct DeleteClanUseCase<R: ClanRepository, L: LeaderboardCache> {
    repo: R,
    leaderboard: L,
}

impl<R: ClanRepository, L: LeaderboardCache> DeleteClanUseCase<R, L> {
    pub fn new(repo: R, leaderboard: L) -> Self {
        Self { repo, leaderboard }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, clan_id: Uuid, dto: DeleteClanDto) -> Result<(), LeagueError> {
        let clan = self
            .repo
            .get_clan_by_id(clan_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        let clan = clan.ok_or_else(|| LeagueError::ClanNotFound(clan_id.to_string()))?;
        if clan.leader_id() != dto.caller_id {
            return Err(LeagueError::NotLeader(
                "Only the clan leader can delete the clan".to_string(),
            ));
        }
        self.repo
            .delete_clan(clan_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        let _ = self.leaderboard.remove_clan_from_leaderboard(clan_id).await;
        Ok(())
    }
}
