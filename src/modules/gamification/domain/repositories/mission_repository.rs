use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::modules::gamification::domain::entities::daily_mission::DailyMission;
use crate::modules::gamification::domain::entities::user_mission::UserMission;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MissionRepository: Send + Sync {
    async fn get_active_missions_by_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<DailyMission>, String>;
    async fn get_user_mission(
        &self,
        user_id: Uuid,
        mission_id: Uuid,
    ) -> Result<Option<UserMission>, String>;
    async fn save_user_mission(&self, user_mission: &UserMission) -> Result<(), String>;
    async fn get_daily_mission_by_id(&self, id: Uuid) -> Result<Option<DailyMission>, String>;
    async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String>;
}
