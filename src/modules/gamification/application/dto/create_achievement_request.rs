use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateAchievementRequestDto {
    pub name: String,
    pub milestone_target: i32,
    pub achievement_type: String,
    pub reward_points: i32,
}