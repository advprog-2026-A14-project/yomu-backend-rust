use crate::modules::league::application::dto::user_tier_dto::UserTierDto;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::shared::domain::base_error::AppError;
use uuid::Uuid;

pub struct GetUserTierUseCase<R: ClanRepository> {
    repository: R,
}

impl<R: ClanRepository> GetUserTierUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<UserTierDto, AppError> {
        let clan_id = self.repository.get_user_clan_id(user_id).await?;

        let Some(clan_id) = clan_id else {
            return Ok(UserTierDto::not_in_clan(user_id));
        };

        let clan = self.repository.get_clan_by_id(clan_id).await?;

        match clan {
            Some(c) => Ok(UserTierDto::from_clan(
                user_id,
                c.id(),
                c.name().to_string(),
                c.tier().to_string(),
            )),
            None => Ok(UserTierDto::not_in_clan(user_id)),
        }
    }
}
