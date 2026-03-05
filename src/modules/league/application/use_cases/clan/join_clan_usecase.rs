use crate::modules::league::application::dto::JoinClanDto;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::modules::league::domain::entities::clan_member::MemberRole;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::shared::domain::base_error::AppError;

pub struct JoinClanUseCase<R: ClanRepository> {
    repo: R,
}

impl<R: ClanRepository> JoinClanUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, dto: JoinClanDto) -> Result<ClanMember, AppError> {
        let clan = self.repo.get_clan_by_id(dto.clan_id).await?;
        if clan.is_none() {
            return Err(AppError::NotFound("Clan not found".to_string()));
        }

        if self.repo.is_user_in_any_clan(dto.user_id).await? {
            return Err(AppError::BadRequest(
                "User is already in a clan".to_string(),
            ));
        }

        let member = ClanMember::new(dto.clan_id, dto.user_id, MemberRole::Member);
        self.repo.add_member(&member).await?;

        Ok(member)
    }
}
