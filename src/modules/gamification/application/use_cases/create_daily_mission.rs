use std::sync::Arc;
use uuid::Uuid;

use crate::modules::gamification::application::dto::{
    DailyMissionAdminRequestDto, DailyMissionItemDto,
};
use crate::modules::gamification::domain::entities::daily_mission::{
    DailyMission, MissionType,
};
use crate::modules::gamification::domain::repositories::mission_repository::MissionRepository;

pub struct CreateDailyMissionUseCase {
    repository: Arc<dyn MissionRepository>,
}

impl CreateDailyMissionUseCase {
    pub fn new(repository: Arc<dyn MissionRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        dto: DailyMissionAdminRequestDto,
    ) -> Result<DailyMissionItemDto, String> {
        let mission_type = parse_mission_type(&dto.mission_type)?;

        let mission = DailyMission::new(
            Uuid::new_v4(),
            dto.description,
            dto.target_count,
            dto.date,
            dto.reward_points,
            mission_type,
        )
        .map_err(|e| e.to_string())?;

        self.repository.create_daily_mission(&mission).await?;

        Ok(DailyMissionItemDto {
            mission_id: mission.id(),
            description: mission.description().to_string(),
            target_count: mission.target_count(),
            current_progress: 0,
            is_claimed: false,
            reward_points: mission.reward_points(),
            mission_type: format!("{:?}", mission.mission_type()),
        })
    }
}

fn parse_mission_type(value: &str) -> Result<MissionType, String> {
    match value {
        "ReadArticle" | "read_article" | "READ_ARTICLE" => Ok(MissionType::ReadArticle),
        "Quiz" | "quiz" | "QUIZ" => Ok(MissionType::Quiz),
        "DailyLogin" | "daily_login" | "DAILY_LOGIN" => Ok(MissionType::DailyLogin),
        _ => Err(format!("Tipe misi harian tidak valid: {}", value)),
    }
}