use std::sync::Arc;
use chrono::Utc;

use crate::modules::gamification::application::dto::quiz_sync::SyncQuizHistoryRequestDto;
use crate::modules::gamification::domain::repositories::mission_repository::MissionRepository;
use crate::modules::gamification::domain::repositories::achievement_repository::AchievementRepository;

pub struct SyncQuizGamificationUseCase {
    pub mission_repo: Arc<dyn MissionRepository>,
    pub achievement_repo: Arc<dyn AchievementRepository>,
}

impl SyncQuizGamificationUseCase {
    pub fn new(
        mission_repo: Arc<dyn MissionRepository>,
        achievement_repo: Arc<dyn AchievementRepository>,
    ) -> Self {
        Self { mission_repo, achievement_repo }
    }

    pub async fn execute(&self, payload: SyncQuizHistoryRequestDto) -> Result<(), String> {
        let now = Utc::now();
        let today = now.naive_utc().date();

        let active_missions = self.mission_repo.get_active_missions_by_date(today).await?;
        
        for mission in active_missions {
            // filter dan cek deskripsi atau tipe misi, misal misi baca
            if mission.description().to_lowercase().contains("baca") {
                
                let mut user_mission = match self.mission_repo.get_user_mission(payload.user_id, mission.id()).await? {
                    Some(um) => um,
                    None => {
                        crate::modules::gamification::domain::entities::user_mission::UserMission::new(payload.user_id, mission.id())
                    }
                };

                user_mission.add_progress(1, mission.target_count());
    
                self.mission_repo.save_user_mission(&user_mission).await?;
            }
        }

        let user_achievements = self.achievement_repo.get_user_achievements(payload.user_id).await?;

        for mut user_ach in user_achievements {
            if user_ach.is_completed() {
                continue;
            }

            if let Some(achievement_master) = self.achievement_repo.get_achievement_by_id(user_ach.achievement_id()).await? {
                
                user_ach.add_progress(1, achievement_master.milestone_target(), now);

                // reward otomatis dapat habis selesaikan achievement
                if user_ach.is_completed() {
                    self.achievement_repo.add_user_score(payload.user_id, achievement_master.reward_points()).await?;
                }

                self.achievement_repo.save_user_achievement(&user_ach).await?;
            }
        }

        Ok(())
    }
}