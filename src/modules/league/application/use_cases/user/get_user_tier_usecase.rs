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
        let tier_info = self.repository.get_user_tier_info(user_id).await?;

        match tier_info {
            Some((clan_id, clan_name, tier)) => Ok(UserTierDto::from_clan(
                user_id,
                clan_id,
                clan_name,
                tier.to_string(),
            )),
            None => Ok(UserTierDto::not_in_clan(user_id)),
        }
    }
}
