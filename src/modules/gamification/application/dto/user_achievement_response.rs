use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct UserAchievementItemDto {
    pub achievement_id: Uuid,
    pub name: String,
    pub milestone_target: i32,
    pub current_progress: i32,
    pub is_completed: bool,
    pub is_shown_on_profile: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub achievement_type: String,
    pub reward_points: i32,
}

#[derive(Debug, Serialize)]
pub struct UserAchievementsResponseDto {
    pub user_id: Uuid,
    pub achievements: Vec<UserAchievementItemDto>,
}