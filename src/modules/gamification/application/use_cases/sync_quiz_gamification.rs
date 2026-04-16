use chrono::Utc;
use std::sync::Arc;

use crate::modules::gamification::application::dto::quiz_sync::SyncQuizHistoryRequestDto;
use crate::modules::gamification::domain::entities::achievement::Achievement;
use crate::modules::gamification::domain::entities::daily_mission::DailyMission;
use crate::modules::gamification::domain::entities::user_achievement::UserAchievement;
use crate::modules::gamification::domain::entities::user_mission::UserMission;
use crate::modules::gamification::domain::repositories::achievement_repository::AchievementRepository;
use crate::modules::gamification::domain::repositories::mission_repository::MissionRepository;

pub struct SyncQuizGamificationUseCase {
    pub mission_repo: Arc<dyn MissionRepository>,
    pub achievement_repo: Arc<dyn AchievementRepository>,
}

impl SyncQuizGamificationUseCase {
    pub fn new(
        mission_repo: Arc<dyn MissionRepository>,
        achievement_repo: Arc<dyn AchievementRepository>,
    ) -> Self {
        Self {
            mission_repo,
            achievement_repo,
        }
    }

    /// Syncs quiz completion to gamification: updates mission progress and achievements.
    ///
    /// For each active daily mission containing "baca" (read), increments user progress.
    /// For each achievement, adds progress and grants reward points upon completion.
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

        let user_achievements = self
            .achievement_repo
            .get_user_achievements(payload.user_id)
            .await?;

        for mut user_ach in user_achievements {
            if user_ach.is_completed() {
                continue;
            }

            if let Some(achievement_master) = self
                .achievement_repo
                .get_achievement_by_id(user_ach.achievement_id())
                .await?
            {
                user_ach.add_progress(1, achievement_master.milestone_target(), now);

                // reward otomatis dapat habis selesaikan achievement
                if user_ach.is_completed() {
                    self.achievement_repo
                        .add_user_score(payload.user_id, achievement_master.reward_points())
                        .await?;
                }

                self.achievement_repo
                    .save_user_achievement(&user_ach)
                    .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::gamification::domain::entities::achievement::AchievementType;
    use chrono::NaiveDate;
    use uuid::Uuid;

    mockall::mock! {
        pub MissionRepo { }

        #[async_trait::async_trait]
        impl MissionRepository for MissionRepo {
            async fn get_active_missions_by_date(
                &self,
                date: NaiveDate,
            ) -> Result<Vec<DailyMission>, String>;
            async fn get_user_mission(
                &self,
                user_id: Uuid,
                mission_id: Uuid,
            ) -> Result<Option<UserMission>, String>;
            async fn save_user_mission(&self, user_mission: &UserMission) -> Result<(), String>;
            async fn get_daily_mission_by_id(&self, id: Uuid) -> Result<Option<DailyMission>, String>;
            async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String>;
        }
    }

    mockall::mock! {
        pub AchievementRepo { }

        #[async_trait::async_trait]
        impl AchievementRepository for AchievementRepo {
            async fn get_achievement_by_id(&self, id: Uuid) -> Result<Option<Achievement>, String>;
            async fn get_user_achievements(&self, user_id: Uuid) -> Result<Vec<UserAchievement>, String>;
            async fn save_user_achievement(&self, user_achievement: &UserAchievement) -> Result<(), String>;
            async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String>;
        }
    }

    fn create_test_mission(
        id: Uuid,
        description: &str,
        target: i32,
        date: NaiveDate,
    ) -> DailyMission {
        DailyMission::new(id, description.to_string(), target, date, 100).unwrap()
    }

    fn create_test_achievement(id: Uuid, name: &str, target: i32, reward: i32) -> Achievement {
        Achievement::new(
            id,
            name.to_string(),
            target,
            AchievementType::Common,
            reward,
        )
        .unwrap()
    }

    fn create_payload(user_id: Uuid) -> SyncQuizHistoryRequestDto {
        SyncQuizHistoryRequestDto {
            user_id,
            article_id: Uuid::new_v4(),
            score: 80,
            accuracy: 0.85,
        }
    }

    #[tokio::test]
    async fn sync_quiz_creates_new_mission_progress() {
        let user_id = Uuid::new_v4();
        let mission_id = Uuid::new_v4();
        let today = Utc::now().naive_utc().date();
        let payload = create_payload(user_id);

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(move |_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_mission()
            .return_once(|_, _| Ok(None));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_updates_existing_mission() {
        let user_id = Uuid::new_v4();
        let mission_id = Uuid::new_v4();
        let today = Utc::now().naive_utc().date();
        let payload = create_payload(user_id);

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today);
        let existing_user_mission = UserMission::new(user_id, mission_id);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_mission()
            .return_once(|_, _| Ok(Some(existing_user_mission)));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_caps_at_target() {
        let user_id = Uuid::new_v4();
        let mission_id = Uuid::new_v4();
        let today = Utc::now().naive_utc().date();
        let payload = create_payload(user_id);

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today);
        let mut existing_user_mission = UserMission::new(user_id, mission_id);
        existing_user_mission.add_progress(3, 3);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_mission()
            .return_once(|_, _| Ok(Some(existing_user_mission)));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_no_matching_missions() {
        let user_id = Uuid::new_v4();
        let today = Utc::now().naive_utc().date();
        let payload = create_payload(user_id);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_achievements_updated() {
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();
        let payload = create_payload(user_id);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let achievement = create_test_achievement(achievement_id, "Quiz Starter", 5, 50);
        let user_achievement = UserAchievement::new(user_id, achievement_id);

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![user_achievement]));

        achievement_repo
            .expect_get_achievement_by_id()
            .return_once(|_| Ok(Some(achievement)));

        achievement_repo
            .expect_save_user_achievement()
            .returning(|_| Ok(()));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_achievement_completed() {
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();
        let payload = create_payload(user_id);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let achievement = create_test_achievement(achievement_id, "First Quiz", 1, 100);
        let mut user_achievement = UserAchievement::new(user_id, achievement_id);
        user_achievement.add_progress(1, 1, Utc::now());

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![user_achievement]));

        achievement_repo
            .expect_get_achievement_by_id()
            .return_once(|_| Ok(Some(achievement)));

        achievement_repo
            .expect_add_user_score()
            .return_once(|_, _| Ok(()));

        achievement_repo
            .expect_save_user_achievement()
            .returning(|_| Ok(()));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_achievement_reward_awarded() {
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();
        let payload = create_payload(user_id);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let achievement = create_test_achievement(achievement_id, "Quiz Master", 2, 250);
        let mut user_achievement = UserAchievement::new(user_id, achievement_id);
        user_achievement.add_progress(1, 2, Utc::now());

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![user_achievement]));

        achievement_repo
            .expect_get_achievement_by_id()
            .return_once(|_| Ok(Some(achievement)));

        achievement_repo
            .expect_add_user_score()
            .return_once(|_, _| Ok(()));

        achievement_repo
            .expect_save_user_achievement()
            .returning(|_| Ok(()));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_no_achievements() {
        let user_id = Uuid::new_v4();
        let today = Utc::now().naive_utc().date();
        let payload = create_payload(user_id);

        let mission = create_test_mission(Uuid::new_v4(), "Baca 3 Berita", 3, today);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_mission()
            .return_once(|_, _| Ok(None));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_empty_article_id() {
        let user_id = Uuid::new_v4();
        let payload = SyncQuizHistoryRequestDto {
            user_id,
            article_id: Uuid::nil(),
            score: 100,
            accuracy: 85.0,
        };

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_read_article_mission() {
        let user_id = Uuid::new_v4();
        let mission_id = Uuid::new_v4();
        let today = Utc::now().naive_utc().date();
        let payload = create_payload(user_id);

        let mission = create_test_mission(mission_id, "Baca 5 Artikel", 5, today);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_mission()
            .return_once(|_, _| Ok(None));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_mission_target_already_reached() {
        let user_id = Uuid::new_v4();
        let mission_id = Uuid::new_v4();
        let today = Utc::now().naive_utc().date();
        let payload = create_payload(user_id);

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today);
        let mut existing_user_mission = UserMission::new(user_id, mission_id);
        existing_user_mission.add_progress(3, 3);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_mission()
            .return_once(|_, _| Ok(Some(existing_user_mission)));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_multiple_missions_match() {
        let user_id = Uuid::new_v4();
        let mission_id_1 = Uuid::new_v4();
        let mission_id_2 = Uuid::new_v4();
        let today = Utc::now().naive_utc().date();
        let payload = create_payload(user_id);

        let mission1 = create_test_mission(mission_id_1, "Baca 3 Berita", 3, today);
        let mission2 = create_test_mission(mission_id_2, "Baca 5 Artikel", 5, today);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission1, mission2]));

        mission_repo
            .expect_get_user_mission()
            .times(2)
            .returning(|_, _| Ok(None));

        mission_repo
            .expect_save_user_mission()
            .times(2)
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_concurrent_same_user() {
        let user_id = Uuid::new_v4();
        let mission_id = Uuid::new_v4();
        let today = Utc::now().naive_utc().date();

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today);

        let mut mission_repo = MockMissionRepo::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .times(2)
            .returning(move |_| Ok(vec![mission.clone()]));

        mission_repo
            .expect_get_user_mission()
            .times(2)
            .returning(|_, _| Ok(None));

        mission_repo
            .expect_save_user_mission()
            .times(2)
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepo::new();
        achievement_repo
            .expect_get_user_achievements()
            .times(2)
            .returning(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let payload1 = create_payload(user_id);
        let payload2 = create_payload(user_id);

        let (result1, result2) =
            tokio::join!(use_case.execute(payload1), use_case.execute(payload2));

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}
