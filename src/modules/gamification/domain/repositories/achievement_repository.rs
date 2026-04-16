use async_trait::async_trait;
use uuid::Uuid;

use crate::modules::gamification::domain::entities::achievement::Achievement;
use crate::modules::gamification::domain::entities::user_achievement::UserAchievement;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AchievementRepository: Send + Sync {
    async fn get_all_achievements(&self) -> Result<Vec<Achievement>, String>;
    async fn get_achievement_by_id(&self, id: Uuid) -> Result<Option<Achievement>, String>;
    async fn get_user_achievements(&self, user_id: Uuid) -> Result<Vec<UserAchievement>, String>;
    async fn save_user_achievement(&self, user_achievement: &UserAchievement) -> Result<(), String>;
    async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String>;
}