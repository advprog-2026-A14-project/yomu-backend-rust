use crate::modules::league::application::dto::CreateClanDto;
use crate::modules::league::domain::entities::clan::Clan;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::modules::league::domain::entities::clan_member::MemberRole;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::shared::domain::base_error::AppError;

pub struct CreateClanUseCase<R: ClanRepository> {
    repo: R,
}

impl<R: ClanRepository> CreateClanUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Creates a new clan with the leader automatically as the first member.
    ///
    /// Validates that the leader is not already in any clan before creating.
    /// Returns the created clan with generated ID.
    pub async fn execute(&self, dto: CreateClanDto) -> Result<Clan, AppError> {
        if self.repo.is_user_in_any_clan(dto.leader_id).await? {
            return Err(AppError::BadRequest(
                "User is already in a clan".to_string(),
            ));
        }

        let clan = Clan::new(dto.name, dto.leader_id);
        self.repo.create_clan(&clan).await?;

        let member = ClanMember::new(clan.id(), dto.leader_id, MemberRole::Leader);
        self.repo.add_member(&member).await?;

        Ok(clan)
    }
}
