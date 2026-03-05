use crate::modules::league::domain::entities::clan::Clan;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ClanRepository: Send + Sync {
    async fn create_clan(&self, clan: &Clan) -> Result<(), AppError>;
    async fn get_clan_by_id(&self, clan_id: Uuid) -> Result<Option<Clan>, AppError>;
    async fn add_member(&self, member: &ClanMember) -> Result<(), AppError>;
    async fn is_user_in_any_clan(&self, user_id: Uuid) -> Result<bool, AppError>;
    async fn get_user_clan_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;
    async fn add_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
}
