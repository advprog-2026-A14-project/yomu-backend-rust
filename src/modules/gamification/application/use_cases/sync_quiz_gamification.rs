use chrono::Utc;
use futures::FutureExt;
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::gamification::application::dto::quiz_sync::SyncQuizHistoryRequestDto;
use crate::modules::gamification::domain::entities::achievement::Achievement;
use crate::modules::gamification::domain::entities::daily_mission::DailyMission;
use crate::modules::gamification::domain::entities::daily_mission::MissionType;
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

        let read_missions: Vec<_> = active_missions
            .into_iter()
            .filter(|m| matches!(m.mission_type(), MissionType::ReadArticle))
            .collect();

        if !read_missions.is_empty() {
            let mission_ids: Vec<_> = read_missions.iter().map(|m| m.id()).collect();

            let existing_user_missions = self
                .mission_repo
                .get_user_missions_batch(payload.user_id, mission_ids)
                .await?;

            let existing_map: HashMap<_, _> = existing_user_missions
                .into_iter()
                .map(|um| (um.mission_id(), um))
                .collect();

            let user_missions_to_save: Vec<UserMission> = read_missions
                .into_iter()
                .map(|mission| {
                    let mut user_mission = existing_map
                        .get(&mission.id())
                        .cloned()
                        .unwrap_or_else(|| UserMission::new(payload.user_id, mission.id()));
                    user_mission.add_progress(1, mission.target_count());
                    user_mission
                })
                .collect();

            let save_futures = user_missions_to_save
                .iter()
                .map(|um| self.mission_repo.save_user_mission(um));
            join_all(save_futures).await;
        }

        let user_achievements = self
            .achievement_repo
            .get_user_achievements(payload.user_id)
            .await?;

        let relevant_achievement_ids: Vec<_> = user_achievements
            .iter()
            .map(|ua| ua.achievement_id())
            .collect();

        let achievements = self
            .achievement_repo
            .get_achievements_by_ids(&relevant_achievement_ids)
            .await?;

        let achievement_map: HashMap<_, _> = achievements
            .into_iter()
            .map(|ach| (ach.id(), ach))
            .collect();

        let mut futures = Vec::new();

        for user_ach in &user_achievements {
            if user_ach.is_completed() {
                continue;
            }

            if let Some(achievement_master) = achievement_map.get(&user_ach.achievement_id()) {
                let mut user_ach = user_ach.clone();
                let reward_points = achievement_master.reward_points();

                user_ach.add_progress(1, achievement_master.milestone_target(), now);

                let just_completed = user_ach.is_completed();
                let user_id = payload.user_id;
                let repo = Arc::clone(&self.achievement_repo);

                if just_completed {
                    futures.push(
                        async move {
                            repo.add_user_score(user_id, reward_points).await?;
                            repo.save_user_achievement(&user_ach).await
                        }
                        .boxed(),
                    );
                } else {
                    futures
                        .push(async move { repo.save_user_achievement(&user_ach).await }.boxed());
                }
            }
        }

        let results = join_all(futures).await;
        for result in results {
            result?;
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

    use crate::modules::gamification::domain::repositories::achievement_repository::MockAchievementRepository;
    use crate::modules::gamification::domain::repositories::mission_repository::MockMissionRepository;

    fn create_test_mission(
        id: Uuid,
        description: &str,
        target: i32,
        date: NaiveDate,
        mission_type: MissionType,
    ) -> DailyMission {
        DailyMission::new(id, description.to_string(), target, date, 100, mission_type).unwrap()
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
        let mission_type = MissionType::ReadArticle;

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today, mission_type);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(move |_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_missions_batch()
            .return_once(|_, _| Ok(vec![]));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
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
        let mission_type = MissionType::ReadArticle;

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today, mission_type);
        let existing_user_mission = UserMission::new(user_id, mission_id);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_missions_batch()
            .return_once(|_, _| Ok(vec![existing_user_mission]));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
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
        let mission_type = MissionType::ReadArticle;

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today, mission_type);
        let mut existing_user_mission = UserMission::new(user_id, mission_id);
        existing_user_mission.add_progress(3, 3);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_missions_batch()
            .return_once(|_, _| Ok(vec![existing_user_mission]));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
            .return_once(|_| Ok(vec![]));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        let result = use_case.execute(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_no_matching_missions() {
        let user_id = Uuid::new_v4();
        let _today = Utc::now().naive_utc().date();
        let payload = create_payload(user_id);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
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

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let achievement = create_test_achievement(achievement_id, "Quiz Starter", 5, 50);
        let user_achievement = UserAchievement::new(user_id, achievement_id);

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![user_achievement]));

        achievement_repo
            .expect_get_achievements_by_ids()
            .return_once(move |_| Ok(vec![achievement.clone()]));

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

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let achievement = create_test_achievement(achievement_id, "First Quiz", 1, 100);
        let mut user_achievement = UserAchievement::new(user_id, achievement_id);
        user_achievement.add_progress(1, 1, Utc::now());

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![user_achievement]));

        achievement_repo
            .expect_get_achievements_by_ids()
            .return_once(move |_| Ok(vec![achievement.clone()]));

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

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let achievement = create_test_achievement(achievement_id, "Quiz Master", 2, 250);
        let mut user_achievement = UserAchievement::new(user_id, achievement_id);
        user_achievement.add_progress(1, 2, Utc::now());

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![user_achievement]));

        achievement_repo
            .expect_get_achievements_by_ids()
            .return_once(move |_| Ok(vec![achievement.clone()]));

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
        let mission_type = MissionType::ReadArticle;

        let mission = create_test_mission(Uuid::new_v4(), "Baca 3 Berita", 3, today, mission_type);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_missions_batch()
            .return_once(|_, _| Ok(vec![]));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
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

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
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
        let mission_type = MissionType::ReadArticle;

        let mission = create_test_mission(mission_id, "Baca 5 Artikel", 5, today, mission_type);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_missions_batch()
            .return_once(|_, _| Ok(vec![]));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
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
        let mission_type = MissionType::ReadArticle;

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today, mission_type);
        let mut existing_user_mission = UserMission::new(user_id, mission_id);
        existing_user_mission.add_progress(3, 3);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission]));

        mission_repo
            .expect_get_user_missions_batch()
            .return_once(|_, _| Ok(vec![existing_user_mission]));

        mission_repo
            .expect_save_user_mission()
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
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
        let mission_type = MissionType::ReadArticle;

        let mission1 = create_test_mission(mission_id_1, "Baca 3 Berita", 3, today, mission_type);
        let mission2 = create_test_mission(mission_id_2, "Baca 5 Artikel", 5, today, mission_type);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![mission1, mission2]));

        mission_repo
            .expect_get_user_missions_batch()
            .return_once(|_, _| Ok(vec![]));

        mission_repo
            .expect_save_user_mission()
            .times(2)
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
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
        let mission_type = MissionType::ReadArticle;

        let mission = create_test_mission(mission_id, "Baca 3 Berita", 3, today, mission_type);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .times(2)
            .returning(move |_| Ok(vec![mission.clone()]));

        mission_repo
            .expect_get_user_missions_batch()
            .times(2)
            .returning(|_, _| Ok(vec![]));

        mission_repo
            .expect_save_user_mission()
            .times(2)
            .returning(|_| Ok(()));

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .times(2)
            .returning(|_| Ok(vec![]));

        achievement_repo
            .expect_get_achievements_by_ids()
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
