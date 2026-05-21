use crate::modules::league::domain::entities::clan_join_request::ClanJoinRequest;
use crate::modules::league::domain::entities::clan_join_request::RequestStatus;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ClanJoinRequestRepository: Send + Sync {
    async fn create_request(&self, request: &ClanJoinRequest) -> Result<(), AppError>;
    async fn get_pending_requests_by_clan(
        &self,
        clan_id: Uuid,
    ) -> Result<Vec<ClanJoinRequest>, AppError>;
    async fn get_request_by_id(&self, request_id: Uuid) -> Result<Option<ClanJoinRequest>, AppError>;
    async fn get_pending_request_by_user(
        &self,
        user_id: Uuid,
        clan_id: Uuid,
    ) -> Result<Option<ClanJoinRequest>, AppError>;
    async fn has_pending_request(&self, user_id: Uuid) -> Result<bool, AppError>;
    async fn update_request_status(
        &self,
        request_id: Uuid,
        status: &RequestStatus,
    ) -> Result<(), AppError>;
    async fn delete_request(&self, request_id: Uuid) -> Result<(), AppError>;
    async fn delete_pending_requests_by_user(&self, user_id: Uuid) -> Result<(), AppError>;
}
