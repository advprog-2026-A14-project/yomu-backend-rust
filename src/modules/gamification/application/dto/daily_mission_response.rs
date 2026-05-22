use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DailyMissionItemDto {
    pub mission_id: Uuid,
    pub description: String,
    pub target_count: i32,
    pub current_progress: i32,
    pub is_claimed: bool,
    pub reward_points: i32,
    pub mission_type: String,
}

#[derive(Debug, Serialize)]
pub struct DailyMissionsResponseDto {
    pub missions: Vec<DailyMissionItemDto>,
}