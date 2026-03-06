use async_trait::async_trait;
use uuid::Uuid;
use chrono::NaiveDate;

use crate::modules::gamification::domain::entities::daily_mission::DailyMission;
use crate::modules::gamification::domain::entities::user_mission::UserMission;

#[async_trait]
pub trait MissionRepository: Send + Sync {
    async fn get_active_missions_by_date(&self, date: NaiveDate) -> Result<Vec<DailyMission>, String>;
    async fn get_user_mission(&self, user_id: Uuid, mission_id: Uuid) -> Result<Option<UserMission>, String>;
    async fn save_user_mission(&self, user_mission: &UserMission) -> Result<(), String>;
}