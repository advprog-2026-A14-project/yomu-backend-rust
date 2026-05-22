use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CreateAchievementResponseDto {
    pub achievement_id: Uuid,
    pub name: String,
    pub milestone_target: i32,
    pub achievement_type: String,
    pub trigger_type: String,
    pub reward_points: i32,
}
