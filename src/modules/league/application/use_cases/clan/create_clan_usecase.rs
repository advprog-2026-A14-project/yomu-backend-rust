use crate::modules::league::application::dto::CreateClanDto;
use crate::modules::league::domain::entities::clan::Clan;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::modules::league::domain::entities::clan_member::MemberRole;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::modules::league::domain::repositories::LeaderboardCache;
use crate::shared::domain::base_error::AppError;
use tracing::instrument;

pub struct CreateClanUseCase<R: ClanRepository, L: LeaderboardCache> {
    repo: R,
    leaderboard: L,
}

impl<R: ClanRepository, L: LeaderboardCache> CreateClanUseCase<R, L> {
    pub fn new(repo: R, leaderboard: L) -> Self {
        Self { repo, leaderboard }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, dto: CreateClanDto) -> Result<Clan, AppError> {
        self.repo.ensure_user_exists(dto.leader_id).await?;

        if self.repo.is_user_in_any_clan(dto.leader_id).await? {
            return Err(AppError::BadRequest(
                "User is already in a clan".to_string(),
            ));
        }

        let clan = Clan::new(dto.name, dto.leader_id);
        self.repo.create_clan(&clan).await?;

        let member = ClanMember::new(clan.id(), dto.leader_id, MemberRole::Leader);
        self.repo.add_member(&member).await?;

        let tier_str = match clan.tier() {
            crate::modules::league::domain::entities::clan::ClanTier::Bronze => "Bronze",
            crate::modules::league::domain::entities::clan::ClanTier::Silver => "Silver",
            crate::modules::league::domain::entities::clan::ClanTier::Gold => "Gold",
            crate::modules::league::domain::entities::clan::ClanTier::Diamond => "Diamond",
        };
        self.leaderboard
            .add_clan_to_tier(clan.id(), tier_str)
            .await?;

        Ok(clan)
    }
}
