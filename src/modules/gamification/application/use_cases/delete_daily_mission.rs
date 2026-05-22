use std::sync::Arc;
use uuid::Uuid;

use crate::modules::gamification::domain::repositories::mission_repository::MissionRepository;

pub struct DeleteDailyMissionUseCase {
    repository: Arc<dyn MissionRepository>,
}

impl DeleteDailyMissionUseCase {
    pub fn new(repository: Arc<dyn MissionRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, mission_id: Uuid) -> Result<(), String> {
        let existing = self
            .repository
            .get_daily_mission_by_id(mission_id)
            .await?;

        if existing.is_none() {
            return Err("Data misi harian tidak ditemukan di sistem.".to_string());
        }

        self.repository.delete_daily_mission(mission_id).await
    }
}