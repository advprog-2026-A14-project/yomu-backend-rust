use crate::modules::league::application::dto::BuffProcessResultDto;
use crate::modules::league::domain::entities::clan_buff::ClanBuff;
use crate::modules::league::domain::repositories::clan_buff_repository::ClanBuffRepository;
use crate::modules::league::domain::repositories::clan_repository::ClanRepository;
use crate::shared::domain::base_error::AppError;
use chrono::Utc;
use tracing::instrument;
use uuid::Uuid;

pub struct ProcessBuffsUseCase<CR: ClanRepository, BR: ClanBuffRepository> {
    clan_repo: CR,
    buff_repo: BR,
}

impl<CR: ClanRepository, BR: ClanBuffRepository> ProcessBuffsUseCase<CR, BR> {
    pub fn new(clan_repo: CR, buff_repo: BR) -> Self {
        Self {
            clan_repo,
            buff_repo,
        }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, clan_id: Uuid) -> Result<BuffProcessResultDto, AppError> {
        let members = self.clan_repo.get_members_by_clan_id(clan_id).await?;
        let user_ids: Vec<Uuid> = members.iter().map(|m| m.user_id()).collect();

        let mut buffs_added: Vec<String> = Vec::new();
        let mut buffs_removed: Vec<String> = Vec::new();

        let avg_accuracy = self
            .buff_repo
            .get_avg_accuracy_for_members(&user_ids)
            .await?;
        let mission_completion_rate = self
            .buff_repo
            .get_mission_completion_rate_for_clan(clan_id)
            .await?;

        let productivity_buff_name = "Productivity Buff";
        let low_accuracy_debuff_name = "Low Accuracy Penalty";

        let existing_productivity = self
            .buff_repo
            .get_buff_by_name(clan_id, productivity_buff_name)
            .await?;
        let existing_accuracy_debuff = self
            .buff_repo
            .get_buff_by_name(clan_id, low_accuracy_debuff_name)
            .await?;

        if avg_accuracy < 0.5 {
            if existing_accuracy_debuff
                .as_ref()
                .map_or(true, |b| !b.is_active())
            {
                let expires_at = Utc::now() + chrono::Duration::days(7);
                let debuff = ClanBuff::new_low_accuracy_debuff(clan_id, expires_at);
                self.buff_repo.activate_buff(&debuff).await?;
                buffs_added.push(low_accuracy_debuff_name.to_string());
            }
        } else if existing_accuracy_debuff
            .as_ref()
            .map_or(false, |b| b.is_active())
        {
            self.buff_repo
                .deactivate_buff_by_name(clan_id, low_accuracy_debuff_name)
                .await?;
            buffs_removed.push(low_accuracy_debuff_name.to_string());
        }

        if mission_completion_rate >= 0.5 {
            if existing_productivity
                .as_ref()
                .map_or(true, |b| !b.is_active())
            {
                let expires_at = Utc::now() + chrono::Duration::days(7);
                let buff = ClanBuff::new_productivity_buff(clan_id, expires_at);
                self.buff_repo.activate_buff(&buff).await?;
                buffs_added.push(productivity_buff_name.to_string());
            }
        } else if existing_productivity
            .as_ref()
            .map_or(false, |b| b.is_active())
        {
            self.buff_repo
                .deactivate_buff_by_name(clan_id, productivity_buff_name)
                .await?;
            buffs_removed.push(productivity_buff_name.to_string());
        }

        Ok(BuffProcessResultDto {
            buffs_added,
            buffs_removed,
        })
    }
}
