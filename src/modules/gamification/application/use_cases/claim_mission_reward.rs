use std::sync::Arc;
use uuid::Uuid;

use crate::modules::gamification::domain::repositories::mission_repository::MissionRepository;

pub struct ClaimMissionRewardUseCase {
    pub repository: Arc<dyn MissionRepository>,
}

impl ClaimMissionRewardUseCase {
    pub fn new(repository: Arc<dyn MissionRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid, mission_id: Uuid) -> Result<(), String> {
        let mut user_mission = self.repository.get_user_mission(user_id, mission_id)
            .await?
            .ok_or("Progres misi tidak ditemukan untuk pengguna ini.")?;

        let daily_mission = self.repository.get_daily_mission_by_id(mission_id)
            .await?
            .ok_or("Data misi harian tidak ditemukan di sistem.")?;

        user_mission.claim_reward(daily_mission.target_count())?;

        self.repository.save_user_mission(&user_mission).await?;

        self.repository.add_user_score(user_id, daily_mission.reward_points()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use crate::modules::gamification::domain::repositories::mission_repository::MockMissionRepository;
    use crate::modules::gamification::domain::entities::user_mission::UserMission;
    use crate::modules::gamification::domain::entities::daily_mission::DailyMission;

    #[tokio::test] 
    async fn test_execute_claim_success_adds_score() {
        let user_id = Uuid::new_v4();
        let mission_id = Uuid::new_v4();
        let reward_points = 50;
        let target_count = 3;

        // Bikin data master misi
        let daily_mission = DailyMission::new(
            mission_id, 
            "Baca 3 Artikel".to_string(), 
            target_count, 
            NaiveDate::from_ymd_opt(2026, 3, 6).unwrap(), 
            reward_points
        ).unwrap();

        let mut user_mission = UserMission::new(user_id, mission_id);
        user_mission.add_progress(target_count, target_count);

        let mut mock_repo = MockMissionRepository::new();

        mock_repo.expect_get_user_mission()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(mission_id))
            .times(1)
            .returning(move |_, _| Ok(Some(user_mission.clone())));

        mock_repo.expect_get_daily_mission_by_id()
            .with(mockall::predicate::eq(mission_id))
            .times(1)
            .returning(move |_| Ok(Some(daily_mission.clone())));

        mock_repo.expect_save_user_mission()
            .times(1)
            .returning(|_| Ok(()));

        mock_repo.expect_add_user_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(reward_points))
            .times(1)
            .returning(|_, _| Ok(()));

        let use_case = ClaimMissionRewardUseCase::new(Arc::new(mock_repo));
        let result = use_case.execute(user_id, mission_id).await;

        assert!(result.is_ok(), "Eksekusi Use Case seharusnya berhasil");
    }
}