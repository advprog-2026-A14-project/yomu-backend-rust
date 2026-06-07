use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use mockall::mock;
use std::sync::Arc;
use uuid::Uuid;

use yomu_backend_rust::modules::gamification::application::ClaimMissionRewardUseCase;
use yomu_backend_rust::modules::gamification::application::SyncQuizGamificationUseCase;
use yomu_backend_rust::modules::gamification::application::SyncQuizHistoryRequestDto;
use yomu_backend_rust::modules::gamification::application::use_cases::{
    CreateAchievementUseCase, CreateDailyMissionUseCase, DeleteDailyMissionUseCase,
    GetDailyMissionsUseCase, GetUserAchievementsUseCase, UpdateDailyMissionUseCase,
};
use yomu_backend_rust::modules::gamification::application::dto::{
    CreateAchievementRequestDto, DailyMissionAdminRequestDto,
};
use yomu_backend_rust::modules::gamification::domain::entities::achievement::{
    Achievement, AchievementTriggerType, AchievementType,
};
use yomu_backend_rust::modules::gamification::domain::entities::daily_mission::{
    DailyMission, MissionType,
};
use yomu_backend_rust::modules::gamification::domain::entities::user_achievement::UserAchievement;
use yomu_backend_rust::modules::gamification::domain::entities::user_mission::UserMission;
use yomu_backend_rust::modules::gamification::domain::repositories::achievement_repository::AchievementRepository;
use yomu_backend_rust::modules::gamification::domain::repositories::mission_repository::MissionRepository;

mock! {
    MissionRepo {}
    #[async_trait]
    impl MissionRepository for MissionRepo {
        async fn get_active_missions_by_date(&self, date: NaiveDate) -> Result<Vec<DailyMission>, String>;
        async fn get_user_mission(&self, user_id: Uuid, mission_id: Uuid) -> Result<Option<UserMission>, String>;
        async fn get_user_missions_batch(&self, user_id: Uuid, mission_ids: Vec<Uuid>) -> Result<Vec<UserMission>, String>;
        async fn save_user_mission(&self, user_mission: &UserMission) -> Result<(), String>;
        async fn get_daily_mission_by_id(&self, id: Uuid) -> Result<Option<DailyMission>, String>;
        async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String>;
        async fn create_daily_mission(&self, mission: &DailyMission) -> Result<(), String>;
        async fn update_daily_mission(&self, mission: &DailyMission) -> Result<(), String>;
        async fn delete_daily_mission(&self, id: Uuid) -> Result<(), String>;
    }
}

mock! {
    AchievementRepo {}
    #[async_trait]
    impl AchievementRepository for AchievementRepo {
        async fn get_all_achievements(&self) -> Result<Vec<Achievement>, String>;
        async fn get_achievements_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Achievement>, String>;
        async fn get_achievement_by_id(&self, id: Uuid) -> Result<Option<Achievement>, String>;
        async fn get_user_achievements(&self, user_id: Uuid) -> Result<Vec<UserAchievement>, String>;
        async fn get_user_achievement(&self, user_id: Uuid, achievement_id: Uuid) -> Result<Option<UserAchievement>, String>;
        async fn save_user_achievement(&self, user_achievement: &UserAchievement) -> Result<(), String>;
        async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String>;
        async fn create_achievement(&self, achievement: &Achievement) -> Result<(), String>;
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn make_daily_mission(id: Uuid, target: i32, mission_type: MissionType) -> DailyMission {
    DailyMission::new(
        id,
        "Test Mission".to_string(),
        target,
        NaiveDate::from_ymd_opt(2026, 5, 22).unwrap(),
        50,
        mission_type,
    )
    .unwrap()
}

fn make_achievement(id: Uuid, target: i32, reward: i32) -> Achievement {
    Achievement::new(
        id,
        "Test Achievement".to_string(),
        target,
        AchievementType::Common,
        AchievementTriggerType::QuizComplete,
        reward,
    )
    .unwrap()
}

fn make_sync_payload(user_id: Uuid) -> SyncQuizHistoryRequestDto {
    SyncQuizHistoryRequestDto {
        user_id,
        article_id: Uuid::new_v4(),
        score: 80,
        accuracy: 0.85,
    }
}

fn empty_achievement_repo() -> MockAchievementRepo {
    let mut repo = MockAchievementRepo::new();
    repo.expect_get_user_achievements()
        .return_once(|_| Ok(vec![]));
    repo.expect_get_all_achievements()
        .return_once(|| Ok(vec![]));
    repo
}

// ─── ClaimMissionRewardUseCase ────────────────────────────────────────────────

#[tokio::test]
async fn claim_mission_reward_success() {
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();
    let target = 3;

    let mission = make_daily_mission(mission_id, target, MissionType::ReadArticle);
    let mut user_mission = UserMission::new(user_id, mission_id);
    user_mission.add_progress(target, target);

    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_user_mission()
        .return_once(move |_, _| Ok(Some(user_mission)));
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(move |_| Ok(Some(mission)));
    mock_repo.expect_save_user_mission().return_once(|_| Ok(()));
    mock_repo.expect_add_user_score().return_once(|_, _| Ok(()));

    let result = ClaimMissionRewardUseCase::new(Arc::new(mock_repo))
        .execute(user_id, mission_id)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn claim_mission_reward_user_mission_not_found() {
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();

    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_user_mission()
        .return_once(|_, _| Ok(None));

    let result = ClaimMissionRewardUseCase::new(Arc::new(mock_repo))
        .execute(user_id, mission_id)
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("tidak ditemukan"));
}

#[tokio::test]
async fn claim_mission_reward_daily_mission_not_found() {
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();

    let user_mission = UserMission::new(user_id, mission_id);

    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_user_mission()
        .return_once(move |_, _| Ok(Some(user_mission)));
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(|_| Ok(None));

    let result = ClaimMissionRewardUseCase::new(Arc::new(mock_repo))
        .execute(user_id, mission_id)
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("tidak ditemukan"));
}

#[tokio::test]
async fn claim_mission_reward_target_not_met_returns_error() {
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();
    let target = 5;

    let mission = make_daily_mission(mission_id, target, MissionType::ReadArticle);
    let mut user_mission = UserMission::new(user_id, mission_id);
    user_mission.add_progress(2, target);

    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_user_mission()
        .return_once(move |_, _| Ok(Some(user_mission)));
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(move |_| Ok(Some(mission)));

    let result = ClaimMissionRewardUseCase::new(Arc::new(mock_repo))
        .execute(user_id, mission_id)
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("belum selesai"));
}

#[tokio::test]
async fn claim_mission_reward_already_claimed_returns_error() {
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();
    let target = 3;

    let mission = make_daily_mission(mission_id, target, MissionType::Quiz);
    let mut user_mission = UserMission::new(user_id, mission_id);
    user_mission.add_progress(target, target);
    let _ = user_mission.claim_reward(target);

    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_user_mission()
        .return_once(move |_, _| Ok(Some(user_mission)));
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(move |_| Ok(Some(mission)));

    let result = ClaimMissionRewardUseCase::new(Arc::new(mock_repo))
        .execute(user_id, mission_id)
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("sudah di-claim"));
}

// ─── SyncQuizGamificationUseCase ─────────────────────────────────────────────

#[tokio::test]
async fn sync_quiz_empty_state_succeeds() {
    let user_id = Uuid::new_v4();

    let mut mission_repo = MockMissionRepo::new();
    mission_repo
        .expect_get_active_missions_by_date()
        .return_once(|_| Ok(vec![]));

    let use_case = SyncQuizGamificationUseCase::new(
        Arc::new(mission_repo),
        Arc::new(empty_achievement_repo()),
    );

    assert!(use_case.execute(make_sync_payload(user_id)).await.is_ok());
}

#[tokio::test]
async fn sync_quiz_daily_login_mission_not_triggered() {
    let user_id = Uuid::new_v4();
    let today = chrono::Utc::now().naive_utc().date();

    let daily_login_mission = DailyMission::new(
        Uuid::new_v4(),
        "Login Harian".to_string(),
        1,
        today,
        10,
        MissionType::DailyLogin,
    )
    .unwrap();

    let mut mission_repo = MockMissionRepo::new();
    mission_repo
        .expect_get_active_missions_by_date()
        .return_once(move |_| Ok(vec![daily_login_mission]));
    // get_user_missions_batch must NOT be called because DailyLogin is filtered out

    let use_case = SyncQuizGamificationUseCase::new(
        Arc::new(mission_repo),
        Arc::new(empty_achievement_repo()),
    );

    assert!(use_case.execute(make_sync_payload(user_id)).await.is_ok());
}

#[tokio::test]
async fn sync_quiz_mission_repo_error_propagates() {
    let user_id = Uuid::new_v4();

    let mut mission_repo = MockMissionRepo::new();
    mission_repo
        .expect_get_active_missions_by_date()
        .return_once(|_| Err("DB connection error".to_string()));

    let use_case = SyncQuizGamificationUseCase::new(
        Arc::new(mission_repo),
        Arc::new(empty_achievement_repo()),
    );

    let result = use_case.execute(make_sync_payload(user_id)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn sync_quiz_achievement_repo_error_propagates() {
    let user_id = Uuid::new_v4();

    let mut mission_repo = MockMissionRepo::new();
    mission_repo
        .expect_get_active_missions_by_date()
        .return_once(|_| Ok(vec![]));

    let mut achievement_repo = MockAchievementRepo::new();
    achievement_repo
        .expect_get_user_achievements()
        .return_once(|_| Err("Achievement DB error".to_string()));

    let use_case =
        SyncQuizGamificationUseCase::new(Arc::new(mission_repo), Arc::new(achievement_repo));

    let result = use_case.execute(make_sync_payload(user_id)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn sync_quiz_quiz_type_mission_is_triggered() {
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();
    let today = chrono::Utc::now().naive_utc().date();

    let quiz_mission = make_daily_mission(mission_id, 5, MissionType::Quiz);

    let mut mission_repo = MockMissionRepo::new();
    mission_repo
        .expect_get_active_missions_by_date()
        .return_once(move |_| Ok(vec![quiz_mission]));
    mission_repo
        .expect_get_user_missions_batch()
        .return_once(|_, _| Ok(vec![]));
    mission_repo
        .expect_save_user_mission()
        .return_once(|_| Ok(()));

    let _ = today;
    let use_case = SyncQuizGamificationUseCase::new(
        Arc::new(mission_repo),
        Arc::new(empty_achievement_repo()),
    );

    assert!(use_case.execute(make_sync_payload(user_id)).await.is_ok());
}

#[tokio::test]
async fn sync_quiz_new_achievement_auto_enrolled_and_completed() {
    let user_id = Uuid::new_v4();
    let achievement_id = Uuid::new_v4();

    let achievement = make_achievement(achievement_id, 1, 100);

    let mut mission_repo = MockMissionRepo::new();
    mission_repo
        .expect_get_active_missions_by_date()
        .return_once(|_| Ok(vec![]));

    let mut achievement_repo = MockAchievementRepo::new();
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

    assert!(use_case.execute(make_sync_payload(user_id)).await.is_ok());
}

// ─── CreateAchievementUseCase ────────────────────────────────────────────────

#[tokio::test]
async fn create_achievement_success() {
    let dto = CreateAchievementRequestDto {
        name: "New Achievement".to_string(),
        milestone_target: 10,
        achievement_type: "Rare".to_string(),
        trigger_type: "QuizComplete".to_string(),
        reward_points: 100,
    };
    let mut mock_repo = MockAchievementRepo::new();
    mock_repo.expect_create_achievement().return_once(|_| Ok(()));
    let use_case = CreateAchievementUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.name, "New Achievement");
    assert_eq!(resp.milestone_target, 10);
    assert_eq!(resp.achievement_type, "Rare");
    assert_eq!(resp.trigger_type, "QuizComplete");
    assert_eq!(resp.reward_points, 100);
}

#[tokio::test]
async fn create_achievement_invalid_trigger_type() {
    let dto = CreateAchievementRequestDto {
        name: "Bad Trigger".to_string(),
        milestone_target: 5,
        achievement_type: "Common".to_string(),
        trigger_type: "InvalidTrigger".to_string(),
        reward_points: 50,
    };
    let mock_repo = MockAchievementRepo::new();
    let use_case = CreateAchievementUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Tipe trigger achievement tidak valid"));
}

#[tokio::test]
async fn create_achievement_repo_error() {
    let dto = CreateAchievementRequestDto {
        name: "Repo Fail".to_string(),
        milestone_target: 3,
        achievement_type: "Common".to_string(),
        trigger_type: "ReadArticle".to_string(),
        reward_points: 20,
    };
    let mut mock_repo = MockAchievementRepo::new();
    mock_repo
        .expect_create_achievement()
        .return_once(|_| Err("db error".to_string()));
    let use_case = CreateAchievementUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_err());
}

// ─── CreateDailyMissionUseCase ───────────────────────────────────────────────

#[tokio::test]
async fn create_daily_mission_success() {
    let dto = DailyMissionAdminRequestDto {
        description: "Read 5 articles".to_string(),
        target_count: 5,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 50,
        mission_type: "ReadArticle".to_string(),
    };
    let mut mock_repo = MockMissionRepo::new();
    mock_repo.expect_create_daily_mission().return_once(|_| Ok(()));
    let use_case = CreateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.description, "Read 5 articles");
    assert_eq!(resp.target_count, 5);
    assert_eq!(resp.reward_points, 50);
    assert_eq!(resp.mission_type, "ReadArticle");
    assert_eq!(resp.current_progress, 0);
    assert!(!resp.is_claimed);
}

#[tokio::test]
async fn create_daily_mission_invalid_mission_type() {
    let dto = DailyMissionAdminRequestDto {
        description: "Invalid".to_string(),
        target_count: 1,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 10,
        mission_type: "BadType".to_string(),
    };
    let mock_repo = MockMissionRepo::new();
    let use_case = CreateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Tipe misi harian tidak valid"));
}

// ─── DeleteDailyMissionUseCase ───────────────────────────────────────────────

#[tokio::test]
async fn delete_daily_mission_success() {
    let mission_id = Uuid::new_v4();
    let mission = DailyMission::new(
        mission_id,
        "Test Mission".to_string(),
        1,
        NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        10,
        MissionType::Quiz,
    )
    .unwrap();
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(move |_| Ok(Some(mission)));
    mock_repo
        .expect_delete_daily_mission()
        .return_once(|_| Ok(()));
    let use_case = DeleteDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(mission_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_daily_mission_not_found() {
    let mission_id = Uuid::new_v4();
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(|_| Ok(None));
    let use_case = DeleteDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(mission_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("tidak ditemukan"));
}

// ─── GetDailyMissionsUseCase ─────────────────────────────────────────────────

#[tokio::test]
async fn get_daily_missions_empty() {
    let user_id = Uuid::new_v4();
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_active_missions_by_date()
        .return_once(|_| Ok(vec![]));
    let use_case = GetDailyMissionsUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(user_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().missions.is_empty());
}

#[tokio::test]
async fn get_daily_missions_with_progress() {
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();
    let mission = DailyMission::new(
        mission_id,
        "Quiz Mission".to_string(),
        3,
        NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        20,
        MissionType::Quiz,
    )
    .unwrap();
    let mut user_mission = UserMission::new(user_id, mission_id);
    user_mission.add_progress(2, 3);
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_active_missions_by_date()
        .return_once(move |_| Ok(vec![mission]));
    mock_repo
        .expect_get_user_missions_batch()
        .return_once(move |_, _| Ok(vec![user_mission]));
    let use_case = GetDailyMissionsUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(user_id).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.missions.len(), 1);
    let item = &resp.missions[0];
    assert_eq!(item.mission_id, mission_id);
    assert_eq!(item.current_progress, 2);
    assert!(!item.is_claimed);
    assert_eq!(item.mission_type, "Quiz");
}

// ─── GetUserAchievementsUseCase ──────────────────────────────────────────────

#[tokio::test]
async fn get_user_achievements_empty() {
    let user_id = Uuid::new_v4();
    let mut mock_repo = MockAchievementRepo::new();
    mock_repo
        .expect_get_user_achievements()
        .return_once(|_| Ok(vec![]));
    let use_case = GetUserAchievementsUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(user_id, false).await;
    assert!(result.is_ok());
    assert!(result.unwrap().achievements.is_empty());
}

#[tokio::test]
async fn get_user_achievements_hidden_filtered() {
    let user_id = Uuid::new_v4();
    let achievement_id = Uuid::new_v4();
    let mut ua = UserAchievement::new(user_id, achievement_id);
    ua.add_progress(1, 1, Utc::now());
    let _ = ua.set_shown_on_profile(false);
    let mut mock_repo = MockAchievementRepo::new();
    mock_repo
        .expect_get_user_achievements()
        .return_once(move |_| Ok(vec![ua]));
    let use_case = GetUserAchievementsUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(user_id, false).await;
    assert!(result.is_ok());
    assert!(result.unwrap().achievements.is_empty());
}

#[tokio::test]
async fn get_user_achievements_visible_only() {
    let user_id = Uuid::new_v4();
    let achievement_id = Uuid::new_v4();
    let mut ua = UserAchievement::new(user_id, achievement_id);
    ua.add_progress(1, 1, Utc::now());
    let _ = ua.set_shown_on_profile(true);
    let master = Achievement::new(
        achievement_id,
        "Master Achievement".to_string(),
        1,
        AchievementType::Common,
        AchievementTriggerType::QuizComplete,
        50,
    )
    .unwrap();
    let mut mock_repo = MockAchievementRepo::new();
    mock_repo
        .expect_get_user_achievements()
        .return_once(move |_| Ok(vec![ua]));
    mock_repo
        .expect_get_achievements_by_ids()
        .return_once(move |_| Ok(vec![master]));
    let use_case = GetUserAchievementsUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(user_id, true).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.achievements.len(), 1);
    assert_eq!(resp.achievements[0].name, "Master Achievement");
}

// ─── UpdateDailyMissionUseCase ───────────────────────────────────────────────

#[tokio::test]
async fn update_daily_mission_success() {
    let mission_id = Uuid::new_v4();
    let existing = DailyMission::new(
        mission_id,
        "Old Description".to_string(),
        1,
        NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        10,
        MissionType::Quiz,
    )
    .unwrap();
    let dto = DailyMissionAdminRequestDto {
        description: "New Description".to_string(),
        target_count: 5,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 20,
        mission_type: "ReadArticle".to_string(),
    };
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(move |_| Ok(Some(existing)));
    mock_repo
        .expect_update_daily_mission()
        .return_once(|_| Ok(()));
    let use_case = UpdateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(mission_id, dto).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.description, "New Description");
    assert_eq!(resp.target_count, 5);
    assert_eq!(resp.reward_points, 20);
    assert_eq!(resp.mission_type, "ReadArticle");
}

#[tokio::test]
async fn update_daily_mission_not_found() {
    let mission_id = Uuid::new_v4();
    let dto = DailyMissionAdminRequestDto {
        description: "New Description".to_string(),
        target_count: 5,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 20,
        mission_type: "ReadArticle".to_string(),
    };
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(|_| Ok(None));
    let use_case = UpdateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(mission_id, dto).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("tidak ditemukan"));
}

#[tokio::test]
async fn update_daily_mission_invalid_mission_type() {
    let mission_id = Uuid::new_v4();
    let existing = DailyMission::new(
        mission_id,
        "Old Description".to_string(),
        1,
        NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        10,
        MissionType::Quiz,
    )
    .unwrap();
    let dto = DailyMissionAdminRequestDto {
        description: "Updated".to_string(),
        target_count: 5,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 20,
        mission_type: "InvalidType".to_string(),
    };
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(move |_| Ok(Some(existing)));
    let use_case = UpdateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(mission_id, dto).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Tipe misi harian tidak valid"));
}

#[tokio::test]
async fn update_daily_mission_repo_error() {
    let mission_id = Uuid::new_v4();
    let existing = DailyMission::new(
        mission_id,
        "Old Description".to_string(),
        1,
        NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        10,
        MissionType::Quiz,
    )
    .unwrap();
    let dto = DailyMissionAdminRequestDto {
        description: "Updated".to_string(),
        target_count: 5,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 20,
        mission_type: "ReadArticle".to_string(),
    };
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(move |_| Ok(Some(existing)));
    mock_repo
        .expect_update_daily_mission()
        .return_once(|_| Err("db error".to_string()));
    let use_case = UpdateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(mission_id, dto).await;
    assert!(result.is_err());
}

// ─── Mission Type Variant Tests ─────────────────────────────────────────────

#[tokio::test]
async fn create_daily_mission_quiz_type() {
    let mut mock_repo = MockMissionRepo::new();
    mock_repo.expect_create_daily_mission().return_once(|_| Ok(()));
    let dto = DailyMissionAdminRequestDto {
        description: "Quiz mission".to_string(),
        target_count: 3,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 30,
        mission_type: "Quiz".to_string(),
    };
    let use_case = CreateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().mission_type, "Quiz");
}

#[tokio::test]
async fn create_daily_mission_daily_login_type() {
    let mut mock_repo = MockMissionRepo::new();
    mock_repo.expect_create_daily_mission().return_once(|_| Ok(()));
    let dto = DailyMissionAdminRequestDto {
        description: "Login mission".to_string(),
        target_count: 1,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 10,
        mission_type: "DailyLogin".to_string(),
    };
    let use_case = CreateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().mission_type, "DailyLogin");
}

#[tokio::test]
async fn create_daily_mission_lowercase_mission_type() {
    let mut mock_repo = MockMissionRepo::new();
    mock_repo.expect_create_daily_mission().return_once(|_| Ok(()));
    let dto = DailyMissionAdminRequestDto {
        description: "Read mission".to_string(),
        target_count: 2,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 15,
        mission_type: "read_article".to_string(),
    };
    let use_case = CreateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().mission_type, "ReadArticle");
}

#[tokio::test]
async fn create_daily_mission_uppercase_mission_type() {
    let mut mock_repo = MockMissionRepo::new();
    mock_repo.expect_create_daily_mission().return_once(|_| Ok(()));
    let dto = DailyMissionAdminRequestDto {
        description: "Read mission".to_string(),
        target_count: 2,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 15,
        mission_type: "READ_ARTICLE".to_string(),
    };
    let use_case = CreateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().mission_type, "ReadArticle");
}

#[tokio::test]
async fn create_daily_mission_repo_error() {
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_create_daily_mission()
        .return_once(|_| Err("db error".to_string()));
    let dto = DailyMissionAdminRequestDto {
        description: "Fail mission".to_string(),
        target_count: 1,
        date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        reward_points: 10,
        mission_type: "Quiz".to_string(),
    };
    let use_case = CreateDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn delete_daily_mission_repo_error() {
    let mission_id = Uuid::new_v4();
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_daily_mission_by_id()
        .return_once(|_| Err("db error".to_string()));
    let use_case = DeleteDailyMissionUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(mission_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_daily_missions_repo_error() {
    let user_id = Uuid::new_v4();
    let mut mock_repo = MockMissionRepo::new();
    mock_repo
        .expect_get_active_missions_by_date()
        .return_once(|_| Err("db error".to_string()));
    let use_case = GetDailyMissionsUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(user_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_user_achievements_repo_error() {
    let user_id = Uuid::new_v4();
    let mut mock_repo = MockAchievementRepo::new();
    mock_repo
        .expect_get_user_achievements()
        .return_once(|_| Err("db error".to_string()));
    let use_case = GetUserAchievementsUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(user_id, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn create_achievement_with_read_article_trigger() {
    let dto = CreateAchievementRequestDto {
        name: "Read 10 Articles".to_string(),
        milestone_target: 10,
        achievement_type: "Common".to_string(),
        trigger_type: "ReadArticle".to_string(),
        reward_points: 50,
    };
    let mut mock_repo = MockAchievementRepo::new();
    mock_repo.expect_create_achievement().return_once(|_| Ok(()));
    let use_case = CreateAchievementUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trigger_type, "ReadArticle");
}

#[tokio::test]
async fn create_achievement_with_daily_login_trigger() {
    let dto = CreateAchievementRequestDto {
        name: "Daily Login".to_string(),
        milestone_target: 1,
        achievement_type: "Common".to_string(),
        trigger_type: "DailyLogin".to_string(),
        reward_points: 10,
    };
    let mut mock_repo = MockAchievementRepo::new();
    mock_repo.expect_create_achievement().return_once(|_| Ok(()));
    let use_case = CreateAchievementUseCase::new(Arc::new(mock_repo));
    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().trigger_type, "DailyLogin");
}
