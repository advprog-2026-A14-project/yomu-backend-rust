use crate::modules::league::domain::entities::clan_buff::ClanBuff;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ClanBuffRepository: Send + Sync {
    async fn activate_buff(&self, buff: &ClanBuff) -> Result<(), AppError>;

    async fn deactivate_buffs_for_clan(&self, clan_id: Uuid) -> Result<(), AppError>;

    async fn get_active_buffs(&self, clan_id: Uuid) -> Result<Vec<ClanBuff>, AppError>;

    async fn get_buff_by_name(
        &self,
        clan_id: Uuid,
        name: &str,
    ) -> Result<Option<ClanBuff>, AppError>;

    async fn get_avg_accuracy_for_members(&self, user_ids: &[Uuid]) -> Result<f64, AppError>;

    async fn get_mission_completion_rate_for_clan(&self, clan_id: Uuid) -> Result<f64, AppError>;

    async fn deactivate_buff_by_name(&self, clan_id: Uuid, buff_name: &str)
    -> Result<(), AppError>;

    async fn get_avg_quiz_score_for_members(&self, user_ids: &[Uuid]) -> Result<i64, AppError>;
}
