use super::super::entities::shadow_user::ShadowUser;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_shadow_user(&self, user: &ShadowUser) -> Result<(), AppError>;
}