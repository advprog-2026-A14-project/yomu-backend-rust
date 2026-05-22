use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DailyMissionAdminRequestDto {
    pub description: String,
    pub target_count: i32,
    pub date: NaiveDate,
    pub reward_points: i32,
    pub mission_type: String,
}
