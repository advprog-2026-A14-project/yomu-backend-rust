use std::sync::Arc;
use uuid::Uuid;

use crate::modules::gamification::application::dto::{
    DailyMissionAdminRequestDto, DailyMissionItemDto,
};
use crate::modules::gamification::domain::entities::daily_mission::{
    DailyMission, MissionType,
};
use crate::modules::gamification::domain::repositories::mission_repository::MissionRepository;

pub struct UpdateDailyMissionUseCase {
    repository: Arc<dyn MissionRepository>,
}

impl UpdateDailyMissionUseCase {
    pub fn new(repository: Arc<dyn MissionRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        mission_id: Uuid,
        dto: DailyMissionAdminRequestDto,
    ) -> Result<DailyMissionItemDto, String> {
        let existing = self
            .repository
            .get_daily_mission_by_id(mission_id)
            .await?;

        if existing.is_none() {
            return Err("Data misi harian tidak ditemukan di sistem.".to_string());
        }

        let mission_type = parse_mission_type(&dto.mission_type)?;

        let mission = DailyMission::new(
            mission_id,
            dto.description,
            dto.target_count,
            dto.date,
            dto.reward_points,
            mission_type,
        )
        .map_err(|e| e.to_string())?;

        self.repository.update_daily_mission(&mission).await?;

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