use chrono::Utc;
use futures::FutureExt;
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::gamification::application::dto::quiz_sync::SyncQuizHistoryRequestDto;
use crate::modules::gamification::domain::entities::achievement::Achievement;
use crate::modules::gamification::domain::entities::achievement::AchievementTriggerType;
use crate::modules::gamification::domain::entities::daily_mission::DailyMission;
use crate::modules::gamification::domain::entities::daily_mission::MissionType;
use crate::modules::gamification::domain::entities::user_achievement::UserAchievement;
use crate::modules::gamification::domain::entities::user_mission::UserMission;
use crate::modules::gamification::domain::repositories::achievement_repository::AchievementRepository;
use crate::modules::gamification::domain::repositories::mission_repository::MissionRepository;
use crate::modules::gamification::infrastructure::http::validate_article;

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

        // §7.3 Fault Tolerance: validate article existence with Java Core.
        // If Java Core is down or the call fails, we log a warning and continue
        // processing missions and achievements without category info.
        // We NEVER discard the sync event due to Java unavailability.
        let java_core_url = std::env::var("JAVA_CORE_URL").ok();
        let java_api_key = std::env::var("JAVA_CORE_API_KEY").unwrap_or_default();

        let article_category: Option<String> = if let Some(ref base_url) = java_core_url {
            match validate_article(base_url, &java_api_key, payload.article_id).await {
                Ok(Some(data)) => {
                    tracing::debug!(
                        article_id = %payload.article_id,
                        category = ?data.category_name,
                        "Article validated via Java Core"
                    );
                    data.category_name
                }
                Ok(None) => {
                    tracing::warn!(
                        article_id = %payload.article_id,
                        "Article tidak ditemukan di Java Core DB; lanjut tanpa validasi kategori"
                    );
                    None
                }
                Err(e) => {
                    tracing::warn!(
                        article_id = %payload.article_id,
                        error = %e,
                        "Java Core tidak dapat dijangkau; lanjut tanpa validasi kategori (§7.3)"
                    );
                    None
                }
            }
        } else {
            tracing::debug!("JAVA_CORE_URL tidak dikonfigurasi; lewati validasi artikel");
            None
        };

        // article_category is available here for future category-specific achievement logic.
        let _ = article_category;

        let active_missions = self.mission_repo.get_active_missions_by_date(today).await?;

        // Both ReadArticle and Quiz mission types are triggered by quiz completion,
        // since completing a quiz means the user has read the article.
        let read_missions: Vec<_> = active_missions
            .into_iter()
            .filter(|m| matches!(m.mission_type(), MissionType::ReadArticle | MissionType::Quiz))
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

        let mut user_achievements = self
            .achievement_repo
            .get_user_achievements(payload.user_id)
            .await?;

        let all_achievements = self.achievement_repo.get_all_achievements().await?;

        let tracked_ids: std::collections::HashSet<_> = user_achievements
            .iter()
            .map(|ua| ua.achievement_id())
            .collect();

        for achievement in &all_achievements {
            if !tracked_ids.contains(&achievement.id()) {
                let new_entry = UserAchievement::new(payload.user_id, achievement.id());
                self.achievement_repo
                    .save_user_achievement(&new_entry)
                    .await?;
                user_achievements.push(new_entry);
            }
        }

        let achievement_map: HashMap<_, _> = all_achievements
            .into_iter()
            .map(|ach| (ach.id(), ach))
            .collect();

        let mut futures = Vec::new();

        for user_ach in &user_achievements {
            if user_ach.is_completed() {
                continue;
            }

            if let Some(achievement_master) = achievement_map.get(&user_ach.achievement_id()) {
                // Only increment achievements whose trigger matches this quiz-completion event.
                // DailyLogin achievements are never incremented here.
                let triggered_by_quiz = matches!(
                    achievement_master.trigger_type(),
                    AchievementTriggerType::QuizComplete | AchievementTriggerType::ReadArticle
                );

                if !triggered_by_quiz {
                    continue;
                }

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
    use crate::modules::gamification::domain::entities::achievement::{
        AchievementTriggerType, AchievementType,
    };
    use axum::extract::connect_info::ResponseFuture;
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
            AchievementTriggerType::QuizComplete,
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

    /// No user achievements and no master achievements in DB.
    fn mock_achievement_repo_empty() -> MockAchievementRepository {
        let mut repo = MockAchievementRepository::new();
        repo.expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));
        repo.expect_get_all_achievements()
            .return_once(|| Ok(vec![]));
        repo 
    }

    /// User already tracked; masters returned from get_all_achievements.
    fn mock_achievement_repo_with_progress(
        user_achievements: Vec<UserAchievement>,
        all_achievements: Vec<Achievement>,
    ) -> MockAchievementRepository {
        let mut repo = MockAchievementRepository::new();
        repo.expect_get_user_achievements()
            .return_once(move |_| Ok(user_achievements));
        repo.expect_get_all_achievements()
            .return_once(move || Ok(all_achievements));
        repo
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

        let achievement_repo = mock_achievement_repo_empty();
        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        let achievement_repo = mock_achievement_repo_empty();
        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        let achievement_repo = mock_achievement_repo_empty();
        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_no_matching_missions() {
        let user_id = Uuid::new_v4();
        let payload = create_payload(user_id);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let achievement_repo = mock_achievement_repo_empty();
        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        let mut achievement_repo =
            mock_achievement_repo_with_progress(vec![user_achievement], vec![achievement.clone()]);
        achievement_repo
            .expect_save_user_achievement()
            .returning(|_| Ok(()));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        // Already completed — loop skips; no save/add_score expected
        let achievement_repo = mock_achievement_repo_with_progress(
            vec![user_achievement],
            vec![achievement],
        );

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        let mut achievement_repo =
            mock_achievement_repo_with_progress(vec![user_achievement], vec![achievement]);
        achievement_repo
            .expect_add_user_score()
            .return_once(|_, _| Ok(()));
        achievement_repo
            .expect_save_user_achievement()
            .returning(|_| Ok(()));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_auto_enrolls_new_achievement() {
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();
        let payload = create_payload(user_id);

        let mut mission_repo = MockMissionRepository::new();
        mission_repo
            .expect_get_active_missions_by_date()
            .return_once(|_| Ok(vec![]));

        let achievement = create_test_achievement(achievement_id, "First Reader", 1, 10);

        let mut achievement_repo = MockAchievementRepository::new();
        achievement_repo
            .expect_get_user_achievements()
            .return_once(|_| Ok(vec![]));
        achievement_repo
            .expect_get_all_achievements()
            .return_once(move || Ok(vec![achievement]));
        achievement_repo
            .expect_save_user_achievement()
            .returning(|_| Ok(()));
        achievement_repo
            .expect_add_user_score()
            .return_once(|_, _| Ok(()));

        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        let achievement_repo = mock_achievement_repo_empty();
        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        let achievement_repo = mock_achievement_repo_empty();
        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        let achievement_repo = mock_achievement_repo_empty();
        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        let achievement_repo = mock_achievement_repo_empty();
        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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

        let achievement_repo = mock_achievement_repo_empty();
        let use_case =
            SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

        assert!(use_case.execute(payload).await.is_ok());
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
            .expect_get_all_achievements()
            .times(2)
            .returning(|| Ok(vec![]));

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