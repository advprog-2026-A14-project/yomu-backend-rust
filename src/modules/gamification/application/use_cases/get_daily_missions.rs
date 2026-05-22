use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::modules::gamification::application::dto::{
    DailyMissionItemDto, DailyMissionsResponseDto,
};
use crate::modules::gamification::domain::repositories::mission_repository::MissionRepository;

pub struct GetDailyMissionsUseCase {
    pub repository: Arc<dyn MissionRepository>,
}

impl GetDailyMissionsUseCase {
    pub fn new(repository: Arc<dyn MissionRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<DailyMissionsResponseDto, String> {
        let today = Utc::now().naive_utc().date();
        let missions = self.repository.get_active_missions_by_date(today).await?;

        if missions.is_empty() {
            return Ok(DailyMissionsResponseDto {
                missions: Vec::new(),
            });
        }

        let mission_ids: Vec<_> = missions.iter().map(|m| m.id()).collect();
        let user_missions = self
            .repository
            .get_user_missions_batch(user_id, mission_ids)
            .await?;

        let progress_map: HashMap<_, _> = user_missions
            .into_iter()
            .map(|um| (um.mission_id(), um))
            .collect();

        let items = missions
            .into_iter()
            .map(|mission| {
                let user_mission = progress_map.get(&mission.id());
                let current_progress = user_mission.map(|um| um.current_progress()).unwrap_or(0);
                let is_claimed = user_mission.map(|um| um.is_claimed()).unwrap_or(false);

                DailyMissionItemDto {
                    mission_id: mission.id(),
                    description: mission.description().to_string(),
                    target_count: mission.target_count(),
                    current_progress,
                    is_claimed,
                    reward_points: mission.reward_points(),
                    mission_type: format!("{:?}", mission.mission_type()),
                }
            })
            .collect();

        Ok(DailyMissionsResponseDto { missions: items })
    }
}
