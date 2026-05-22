use crate::modules::league::application::dto::ScoreResultDto;
use crate::modules::league::domain::entities::clan::ClanTier;
use crate::modules::league::domain::entities::scoring_strategy::{
    ScoringStrategy, calculate_score,
};
use crate::modules::league::domain::repositories::ClanBuffRepository;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::modules::league::domain::repositories::LeaderboardCache;
use crate::shared::domain::base_error::AppError;
use tracing::instrument;
use uuid::Uuid;

pub struct UpdateScoreWithBuffsUseCase<
    R: ClanRepository,
    B: ClanBuffRepository,
    L: LeaderboardCache,
> {
    clan_repo: R,
    buff_repo: B,
    leaderboard: L,
}

impl<R: ClanRepository, B: ClanBuffRepository, L: LeaderboardCache>
    UpdateScoreWithBuffsUseCase<R, B, L>
{
    pub fn new(clan_repo: R, buff_repo: B, leaderboard: L) -> Self {
        Self {
            clan_repo,
            buff_repo,
            leaderboard,
        }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, clan_id: Uuid) -> Result<ScoreResultDto, AppError> {
        tracing::info!(%clan_id, "Executing update score with buffs");
        let clan = self
            .clan_repo
            .get_clan_by_id(clan_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Clan not found: {}", clan_id)))?;

        let members = self.clan_repo.get_members_by_clan_id(clan_id).await?;
        let user_ids: Vec<Uuid> = members.iter().map(|m| m.user_id()).collect();

        let active_buffs = self.buff_repo.get_active_buffs(clan_id).await?;

        let combined_multiplier: f64 = active_buffs
            .iter()
            .fold(1.0, |acc, buff| acc * buff.multiplier());

        let strategy = strategy_from_tier(clan.tier());

        let base_score = self
            .buff_repo
            .get_avg_quiz_score_for_members(&user_ids)
            .await?;
        let member_scores: Vec<(Uuid, i64)> = user_ids.iter().map(|&id| (id, base_score)).collect();
        let strategy_score = calculate_score(strategy, &member_scores);

        let final_score = ((strategy_score as f64) * combined_multiplier).round() as i64;

        self.clan_repo.add_score(clan_id, final_score).await?;

        self.leaderboard
            .update_clan_score(clan_id, final_score)
            .await?;

        Ok(ScoreResultDto {
            final_score,
            multiplier: combined_multiplier,
            strategy: strategy.to_string(),
        })
    }
}

fn strategy_from_tier(tier: &ClanTier) -> ScoringStrategy {
    match tier {
        ClanTier::Bronze => ScoringStrategy::BronzeSum,
        ClanTier::Silver => ScoringStrategy::SilverWeightedAvg,
        ClanTier::Gold => ScoringStrategy::GoldWeightedAvg,
        ClanTier::Diamond => ScoringStrategy::DiamondWeightedAvg,
    }
}
