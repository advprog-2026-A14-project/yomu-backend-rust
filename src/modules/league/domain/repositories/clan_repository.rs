use crate::modules::league::domain::entities::clan::Clan;
use crate::modules::league::domain::entities::clan::ClanTier;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ClanRepository: Send + Sync {
    async fn create_clan(&self, clan: &Clan) -> Result<(), AppError>;
    async fn get_clan_by_id(&self, clan_id: Uuid) -> Result<Option<Clan>, AppError>;
    async fn add_member(&self, member: &ClanMember) -> Result<(), AppError>;
    async fn get_members_by_clan_id(&self, clan_id: Uuid) -> Result<Vec<ClanMember>, AppError>;
    async fn is_user_in_any_clan(&self, user_id: Uuid) -> Result<bool, AppError>;
    async fn get_user_clan_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;
    async fn get_user_tier_info(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(Uuid, String, ClanTier)>, AppError>;
    async fn add_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
    async fn delete_clan(&self, clan_id: Uuid) -> Result<(), AppError>;
    async fn get_leaders_by_clan_ids(
        &self,
        clan_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Uuid>, AppError>;
    async fn get_clan_names_by_ids(
        &self,
        clan_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, String>, AppError>;
}
