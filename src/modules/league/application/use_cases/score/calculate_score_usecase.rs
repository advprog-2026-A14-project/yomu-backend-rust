use crate::modules::league::application::dto::UpdateScoreDto;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::modules::league::domain::repositories::LeaderboardCache;
use crate::shared::domain::base_error::AppError;

pub struct UpdateScoreUseCase<R: ClanRepository, L: LeaderboardCache> {
    repo: R,
    leaderboard: L,
}

impl<R: ClanRepository, L: LeaderboardCache> UpdateScoreUseCase<R, L> {
    pub fn new(repo: R, leaderboard: L) -> Self {
        Self { repo, leaderboard }
    }

    pub async fn execute(&self, dto: UpdateScoreDto) -> Result<i64, AppError> {
        let final_score = (dto.base_score as f64 * dto.multiplier) as i64;

        self.repo.add_score(dto.clan_id, final_score).await?;

        self.leaderboard
            .update_clan_score(dto.clan_id, final_score)
            .await?;

        Ok(final_score)
    }
}
