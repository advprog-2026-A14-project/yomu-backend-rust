use std::collections::HashSet;

use tracing::instrument;
use uuid::Uuid;

use crate::modules::league::application::dto::season_result_dto::{
    ClanDemotion, ClanPromotion, SeasonResultDto,
};
use crate::modules::league::domain::entities::clan::ClanTier;
use crate::modules::league::domain::repositories::season_repository::SeasonRepository;
use crate::shared::domain::base_error::AppError;

const PROMOTION_COUNT: usize = 3;
const DEMOTION_COUNT: usize = 3;

fn next_tier(current: &ClanTier) -> Option<ClanTier> {
    match current {
        ClanTier::Bronze => Some(ClanTier::Silver),
        ClanTier::Silver => Some(ClanTier::Gold),
        ClanTier::Gold => Some(ClanTier::Diamond),
        ClanTier::Diamond => None,
    }
}

fn prev_tier(current: &ClanTier) -> Option<ClanTier> {
    match current {
        ClanTier::Silver => Some(ClanTier::Bronze),
        ClanTier::Gold => Some(ClanTier::Silver),
        ClanTier::Diamond => Some(ClanTier::Gold),
        ClanTier::Bronze => None,
    }
}

pub struct TriggerSeasonEndUseCase<R: SeasonRepository> {
    repo: R,
}

impl<R: SeasonRepository> TriggerSeasonEndUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, season_id: Uuid) -> Result<SeasonResultDto, AppError> {
        let season = self
            .repo
            .get_season_by_id(season_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Season not found: {}", season_id)))?;

        let tier = *season.tier();

        let results = self.repo.get_season_results(&tier, season_id).await?;

        let mut promoted: Vec<ClanPromotion> = Vec::new();
        let mut demoted: Vec<ClanDemotion> = Vec::new();
        let mut promoted_ids: HashSet<Uuid> = HashSet::new();

        if let Some(to_tier) = next_tier(&tier) {
            for i in 0..PROMOTION_COUNT.min(results.len()) {
                let (clan_id, ref clan_name, rank, score) = results[i];
                promoted_ids.insert(clan_id);
                promoted.push(ClanPromotion {
                    clan_id,
                    clan_name: clan_name.clone(),
                    from_tier: tier,
                    to_tier,
                    final_rank: rank,
                    final_score: score,
                });
                self.repo.update_clan_tier(clan_id, &to_tier).await?;
            }
        }

        if let Some(to_tier) = prev_tier(&tier) {
            let len = results.len();
            let demote_start = len.saturating_sub(DEMOTION_COUNT);

            for &(clan_id, ref clan_name, rank, score) in &results[demote_start..] {
                if promoted_ids.contains(&clan_id) {
                    continue;
                }
                demoted.push(ClanDemotion {
                    clan_id,
                    clan_name: clan_name.clone(),
                    from_tier: tier,
                    to_tier,
                    final_rank: rank,
                    final_score: score,
                });
                self.repo.update_clan_tier(clan_id, &to_tier).await?;
            }
        }

        self.repo.mark_season_ended(season_id).await?;

        Ok(SeasonResultDto {
            season_id,
            promoted,
            demoted,
        })
    }
}
