use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

use yomu_backend_rust::modules::league::application::ApproveJoinRequestUseCase;
use yomu_backend_rust::modules::league::application::CreateClanUseCase;
use yomu_backend_rust::modules::league::application::CreateJoinRequestUseCase;
use yomu_backend_rust::modules::league::application::DeleteClanUseCase;
use yomu_backend_rust::modules::league::application::GetClanBuffsUseCase;
use yomu_backend_rust::modules::league::application::GetClanDetailUseCase;
use yomu_backend_rust::modules::league::application::GetLeaderboardUseCase;
use yomu_backend_rust::modules::league::application::GetPendingRequestsUseCase;
use yomu_backend_rust::modules::league::application::GetUserTierUseCase;
use yomu_backend_rust::modules::league::application::JoinClanUseCase;
use yomu_backend_rust::modules::league::application::ProcessBuffsUseCase;
use yomu_backend_rust::modules::league::application::RejectJoinRequestUseCase;
use yomu_backend_rust::modules::league::application::UpdateScoreWithBuffsUseCase;
use yomu_backend_rust::modules::league::application::dto::ApproveRejectDto;
use yomu_backend_rust::modules::league::application::dto::BuffProcessResultDto;
use yomu_backend_rust::modules::league::application::dto::ClanBuffsDto;
use yomu_backend_rust::modules::league::application::dto::CreateClanDto;
use yomu_backend_rust::modules::league::application::dto::CreateJoinRequestDto;
use yomu_backend_rust::modules::league::application::dto::DeleteClanDto;
use yomu_backend_rust::modules::league::application::dto::JoinClanDto;
use yomu_backend_rust::modules::league::application::dto::LeaderboardEntry;
use yomu_backend_rust::modules::league::application::dto::ScoreResultDto;
use yomu_backend_rust::modules::league::application::dto::SeasonResultDto;
use yomu_backend_rust::modules::league::application::dto::UserTierDto;
use yomu_backend_rust::modules::league::application::use_cases::TriggerSeasonEndUseCase;
use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
use yomu_backend_rust::modules::league::domain::entities::clan::ClanTier;
use yomu_backend_rust::modules::league::domain::entities::clan_buff::ClanBuff;
use yomu_backend_rust::modules::league::domain::entities::clan_join_request::ClanJoinRequest;
use yomu_backend_rust::modules::league::domain::entities::clan_join_request::RequestStatus;
use yomu_backend_rust::modules::league::domain::entities::clan_member::ClanMember;
use yomu_backend_rust::modules::league::domain::entities::clan_member::MemberRole;
use yomu_backend_rust::modules::league::domain::entities::season::Season;
use yomu_backend_rust::modules::league::domain::errors::LeagueError;
use yomu_backend_rust::modules::league::domain::repositories::ClanBuffRepository;
use yomu_backend_rust::modules::league::domain::repositories::ClanJoinRequestRepository;
use yomu_backend_rust::modules::league::domain::repositories::ClanRepository;
use yomu_backend_rust::modules::league::domain::repositories::LeaderboardCache;
use yomu_backend_rust::modules::league::domain::repositories::season_repository::SeasonRepository;
use yomu_backend_rust::shared::domain::base_error::AppError;

mock! {
    ClanRepositoryRepo {}
    #[async_trait]
    impl ClanRepository for ClanRepositoryRepo {
        async fn create_clan(&self, clan: &Clan) -> Result<(), AppError>;
        async fn get_clan_by_id(&self, clan_id: Uuid) -> Result<Option<Clan>, AppError>;
        async fn add_member(&self, member: &ClanMember) -> Result<(), AppError>;
        async fn get_members_by_clan_id(&self, clan_id: Uuid) -> Result<Vec<ClanMember>, AppError>;
        async fn is_user_in_any_clan(&self, user_id: Uuid) -> Result<bool, AppError>;
        async fn get_user_clan_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;
        async fn add_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
        async fn delete_clan(&self, clan_id: Uuid) -> Result<(), AppError>;
        async fn get_user_tier_info(&self, user_id: Uuid) -> Result<Option<(Uuid, String, ClanTier)>, AppError>;
        async fn get_leaders_by_clan_ids(
            &self,
            clan_ids: &[Uuid],
        ) -> Result<std::collections::HashMap<Uuid, Uuid>, AppError>;
        async fn get_clan_names_by_ids(
            &self,
            clan_ids: &[Uuid],
        ) -> Result<std::collections::HashMap<Uuid, String>, AppError>;
    }
}

mock! {
    ClanBuffRepo {}
    #[async_trait]
    impl ClanBuffRepository for ClanBuffRepo {
        async fn activate_buff(&self, buff: &ClanBuff) -> Result<(), AppError>;
        async fn deactivate_buffs_for_clan(&self, clan_id: Uuid) -> Result<(), AppError>;
        async fn get_active_buffs(&self, clan_id: Uuid) -> Result<Vec<ClanBuff>, AppError>;
        async fn get_buff_by_name(&self, clan_id: Uuid, name: &str) -> Result<Option<ClanBuff>, AppError>;
        async fn get_avg_accuracy_for_members(&self, user_ids: &[Uuid]) -> Result<f64, AppError>;
        async fn get_mission_completion_rate_for_clan(&self, clan_id: Uuid) -> Result<f64, AppError>;
        async fn deactivate_buff_by_name(&self, clan_id: Uuid, buff_name: &str) -> Result<(), AppError>;
        async fn get_avg_quiz_score_for_members(&self, user_ids: &[Uuid]) -> Result<i64, AppError>;
    }
}

mock! {
    LeaderboardCacheRepo {}
    #[async_trait]
    impl LeaderboardCache for LeaderboardCacheRepo {
        async fn update_clan_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
        async fn add_clan_to_tier(&self, clan_id: Uuid, tier: &str) -> Result<(), AppError>;
        async fn get_top_clans(&self, tier: &str, limit: usize) -> Result<Vec<LeaderboardEntry>, AppError>;
        async fn get_clan_score(&self, clan_id: Uuid) -> Result<Option<i64>, AppError>;
        async fn remove_clan_from_leaderboard(&self, clan_id: Uuid) -> Result<(), AppError>;
    }
}

mock! {
    SeasonRepo {}
    #[async_trait]
    impl SeasonRepository for SeasonRepo {
        async fn create_season(&self, season: &Season) -> Result<(), AppError>;
        async fn get_active_season(&self, tier: &ClanTier) -> Result<Option<Season>, AppError>;
        async fn get_season_by_id(&self, season_id: Uuid) -> Result<Option<Season>, AppError>;
        async fn get_season_results(&self, tier: &ClanTier, season_id: Uuid) -> Result<Vec<(Uuid, String, i64, i64)>, AppError>;
        async fn mark_season_ended(&self, season_id: Uuid) -> Result<(), AppError>;
        async fn update_clan_tier(&self, clan_id: Uuid, new_tier: &ClanTier) -> Result<(), AppError>;
    }
}

mock! {
    ClanJoinRequestRepo {}
    #[async_trait]
    impl ClanJoinRequestRepository for ClanJoinRequestRepo {
        async fn create_request(&self, request: &ClanJoinRequest) -> Result<(), AppError>;
        async fn get_pending_requests_by_clan(&self, clan_id: Uuid) -> Result<Vec<ClanJoinRequest>, AppError>;
        async fn get_request_by_id(&self, request_id: Uuid) -> Result<Option<ClanJoinRequest>, AppError>;
        async fn get_pending_request_by_user(&self, user_id: Uuid, clan_id: Uuid) -> Result<Option<ClanJoinRequest>, AppError>;
        async fn has_pending_request(&self, user_id: Uuid) -> Result<bool, AppError>;
        async fn update_request_status(&self, request_id: Uuid, status: &RequestStatus) -> Result<(), AppError>;
        async fn delete_request(&self, request_id: Uuid) -> Result<(), AppError>;
        async fn delete_pending_requests_by_user(&self, user_id: Uuid) -> Result<(), AppError>;
    }
}

// ============================================================
// CreateClanUseCase Tests
// ============================================================

#[tokio::test]
async fn test_create_clan_success() {
    let leader_id = Uuid::new_v4();
    let clan_name = "Test Clan";

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_is_user_in_any_clan()
        .with(mockall::predicate::eq(leader_id))
        .return_once(|_| Ok(false))
        .once();

    mock_repo
        .expect_create_clan()
        .return_once(|_| Ok(()))
        .once();

    mock_repo.expect_add_member().return_once(|_| Ok(())).once();

    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();
    mock_leaderboard
        .expect_add_clan_to_tier()
        .return_once(|_, _| Ok(()))
        .once();

    let use_case = CreateClanUseCase::new(mock_repo, mock_leaderboard);
    let dto = CreateClanDto {
        name: clan_name.to_string(),
        leader_id,
    };

    let result = use_case.execute(dto).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}

#[tokio::test]
async fn test_create_clan_leader_already_in_clan() {
    let leader_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_is_user_in_any_clan()
        .with(mockall::predicate::eq(leader_id))
        .return_once(|_| Ok(true))
        .once();

    let use_case = CreateClanUseCase::new(mock_repo, MockLeaderboardCacheRepo::new());
    let dto = CreateClanDto {
        name: "Test Clan".to_string(),
        leader_id,
    };

    let result = use_case.execute(dto).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
}

#[tokio::test]
async fn create_clan_empty_name() {
    let leader_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));
    mock_repo.expect_create_clan().return_once(|_| Ok(()));
    mock_repo.expect_add_member().return_once(|_| Ok(()));

    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();
    mock_leaderboard
        .expect_add_clan_to_tier()
        .return_once(|_, _| Ok(()))
        .once();

    let use_case = CreateClanUseCase::new(mock_repo, mock_leaderboard);
    let dto = CreateClanDto {
        name: "".to_string(),
        leader_id,
    };

    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_clan_name_too_long() {
    let leader_id = Uuid::new_v4();
    let long_name = "a".repeat(51);

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));

    mock_repo.expect_create_clan().return_once(|_| Ok(()));
    mock_repo.expect_add_member().return_once(|_| Ok(()));

    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();
    mock_leaderboard
        .expect_add_clan_to_tier()
        .return_once(|_, _| Ok(()))
        .once();

    let use_case = CreateClanUseCase::new(mock_repo, mock_leaderboard);
    let dto = CreateClanDto {
        name: long_name,
        leader_id,
    };

    let result = use_case.execute(dto).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn create_clan_concurrent_race() {
    let leader_id = Uuid::new_v4();
    let clan_name = "Race Clan";

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false))
        .once();

    mock_repo
        .expect_create_clan()
        .return_once(|_| Ok(()))
        .once();
    mock_repo.expect_add_member().return_once(|_| Ok(())).once();

    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();
    mock_leaderboard
        .expect_add_clan_to_tier()
        .return_once(|_, _| Ok(()))
        .once();

    let use_case = CreateClanUseCase::new(mock_repo, mock_leaderboard);
    let dto = CreateClanDto {
        name: clan_name.to_string(),
        leader_id,
    };

    let result = use_case.execute(dto).await;
    assert!(result.is_ok());

    let mut mock_repo2 = MockClanRepositoryRepo::new();

    mock_repo2
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(true))
        .once();

    let use_case2 = CreateClanUseCase::new(mock_repo2, MockLeaderboardCacheRepo::new());
    let dto2 = CreateClanDto {
        name: clan_name.to_string(),
        leader_id,
    };

    let result2 = use_case2.execute(dto2).await;
    assert!(result2.is_err());
    let err = result2.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
}

#[tokio::test]
async fn create_clan_repo_error() {
    let leader_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));

    mock_repo
        .expect_create_clan()
        .return_once(|_| Err(AppError::InternalServer("DB error".to_string())));

    let use_case = CreateClanUseCase::new(mock_repo, MockLeaderboardCacheRepo::new());
    let dto = CreateClanDto {
        name: "Test Clan".to_string(),
        leader_id,
    };

    let result = use_case.execute(dto).await;

    assert!(result.is_err());
}

// ============================================================
// JoinClanUseCase Tests
// ============================================================

#[tokio::test]
async fn test_join_clan_success() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(Some(clan)))
        .once();

    mock_repo
        .expect_is_user_in_any_clan()
        .with(mockall::predicate::eq(user_id))
        .return_once(|_| Ok(false))
        .once();

    mock_repo.expect_add_member().return_once(|_| Ok(())).once();

    let use_case = JoinClanUseCase::new(mock_repo);
    let dto = JoinClanDto { clan_id, user_id };

    let result = use_case.execute(dto).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}

#[tokio::test]
async fn test_join_clan_not_found() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(None))
        .once();

    let use_case = JoinClanUseCase::new(mock_repo);
    let dto = JoinClanDto { clan_id, user_id };

    let result = use_case.execute(dto).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn join_clan_user_already_in_clan() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));

    mock_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(true));

    let use_case = JoinClanUseCase::new(mock_repo);
    let dto = JoinClanDto { clan_id, user_id };

    let result = use_case.execute(dto).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(msg) if msg.contains("already in a clan")));
}

#[tokio::test]
async fn join_clan_user_not_in_any_clan_case() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Silver,
        100,
        chrono::Utc::now(),
    );

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));

    mock_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));

    mock_repo.expect_add_member().return_once(|_| Ok(()));

    let use_case = JoinClanUseCase::new(mock_repo);
    let dto = JoinClanDto { clan_id, user_id };

    let result = use_case.execute(dto).await;

    assert!(result.is_ok());
    let member = result.unwrap();
    assert_eq!(member.user_id(), user_id);
    assert_eq!(member.clan_id(), clan_id);
    assert_eq!(member.role(), &MemberRole::Member);
}

#[tokio::test]
async fn join_clan_concurrent_join() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Concurrent Clan".to_string(),
        leader_id,
        ClanTier::Gold,
        500,
        chrono::Utc::now(),
    );

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));

    mock_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));

    mock_repo.expect_add_member().return_once(|_| Ok(()));

    let use_case = JoinClanUseCase::new(mock_repo);
    let dto = JoinClanDto { clan_id, user_id };

    let result = use_case.execute(dto).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn join_clan_repo_error() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));

    mock_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));

    mock_repo
        .expect_add_member()
        .return_once(|_| Err(AppError::InternalServer("DB error".to_string())));

    let use_case = JoinClanUseCase::new(mock_repo);
    let dto = JoinClanDto { clan_id, user_id };

    let result = use_case.execute(dto).await;

    assert!(result.is_err());
}

// ============================================================
// GetClanDetailUseCase Tests
// ============================================================

#[tokio::test]
async fn get_clan_detail_success() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let member1_id = Uuid::new_v4();
    let member2_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Detail Clan".to_string(),
        leader_id,
        ClanTier::Diamond,
        1500,
        chrono::Utc::now(),
    );

    let members = vec![
        ClanMember::with_joined_at(clan_id, leader_id, MemberRole::Leader, chrono::Utc::now()),
        ClanMember::with_joined_at(clan_id, member1_id, MemberRole::Member, chrono::Utc::now()),
        ClanMember::with_joined_at(clan_id, member2_id, MemberRole::Member, chrono::Utc::now()),
    ];

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));

    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(|_| Ok(members));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_active_buffs()
        .return_once(|_| Ok(vec![]));

    let use_case = GetClanDetailUseCase::new(mock_repo, mock_buff_repo);

    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.id, clan_id);
    assert_eq!(dto.name, "Detail Clan");
    assert_eq!(dto.leader_id, leader_id);
    assert_eq!(dto.tier, "Diamond");
    assert_eq!(dto.total_score, 1500);
    assert_eq!(dto.members.len(), 3);
}

#[tokio::test]
async fn get_clan_detail_not_found() {
    let clan_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo.expect_get_clan_by_id().return_once(|_| Ok(None));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_active_buffs()
        .return_once(|_| Ok(vec![]));

    let use_case = GetClanDetailUseCase::new(mock_repo, mock_buff_repo);

    let result = use_case.execute(clan_id).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn get_clan_detail_empty_clan() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Empty Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));

    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(|_| Ok(vec![]));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_active_buffs()
        .return_once(|_| Ok(vec![]));

    let use_case = GetClanDetailUseCase::new(mock_repo, mock_buff_repo);

    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.id, clan_id);
    assert_eq!(dto.members.len(), 0);
}

#[tokio::test]
async fn clan_detail_with_db_error() {
    let clan_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Err(AppError::InternalServer("DB connection failed".to_string())));

    let use_case = GetClanDetailUseCase::new(mock_repo, MockClanBuffRepo::new());

    let result = use_case.execute(clan_id).await;

    assert!(result.is_err());
}

// ============================================================
// GetUserTierUseCase Tests
// ============================================================

#[tokio::test]
async fn get_user_tier_has_clan() {
    let user_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let _leader_id = Uuid::new_v4();

    let tier_info = (clan_id, "Tier Clan".to_string(), ClanTier::Gold);

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_user_tier_info()
        .return_once(move |_| Ok(Some(tier_info)));

    let use_case = GetUserTierUseCase::new(mock_repo);

    let result: Result<UserTierDto, AppError> = use_case.execute(user_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.user_id, user_id);
    assert_eq!(dto.clan_id, Some(clan_id));
    assert_eq!(dto.clan_name, Some("Tier Clan".to_string()));
    assert_eq!(dto.tier, Some("Gold".to_string()));
}

#[tokio::test]
async fn get_user_tier_no_clan() {
    let user_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_user_tier_info()
        .return_once(|_| Ok(None));

    let use_case = GetUserTierUseCase::new(mock_repo);

    let result: Result<UserTierDto, AppError> = use_case.execute(user_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.user_id, user_id);
    assert!(dto.clan_id.is_none());
    assert!(dto.clan_name.is_none());
    assert!(dto.tier.is_none());
}

#[tokio::test]
async fn get_user_tier_empty_optionals() {
    let user_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_user_tier_info()
        .return_once(|_| Ok(None));

    let use_case = GetUserTierUseCase::new(mock_repo);

    let result: Result<UserTierDto, AppError> = use_case.execute(user_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.user_id, user_id);
    assert!(dto.clan_id.is_none());
    assert!(dto.clan_name.is_none());
    assert!(dto.tier.is_none());
}

#[tokio::test]
async fn empty_database_get_user_tier() {
    let user_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_user_tier_info()
        .return_once(|_| Ok(None));

    let use_case = GetUserTierUseCase::new(mock_repo);

    let result: Result<UserTierDto, AppError> = use_case.execute(user_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert!(dto.clan_id.is_none());
}

// ============================================================
// GetLeaderboardUseCase Tests
// ============================================================

#[tokio::test]
async fn test_get_leaderboard_success() {
    let tier = "Diamond".to_string();
    let clan_id_a = Uuid::new_v4();
    let clan_id_b = Uuid::new_v4();
    let leader_a = Uuid::new_v4();
    let leader_b = Uuid::new_v4();
    let entries = vec![
        LeaderboardEntry {
            clan_id: clan_id_a,
            clan_name: "Clan A".to_string(),
            leader_id: Uuid::nil(),
            total_score: 1000,
            tier: tier.clone(),
            rank: 1,
        },
        LeaderboardEntry {
            clan_id: clan_id_b,
            clan_name: "Clan B".to_string(),
            leader_id: Uuid::nil(),
            total_score: 800,
            tier: tier.clone(),
            rank: 2,
        },
    ];

    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();
    mock_leaderboard
        .expect_get_top_clans()
        .with(
            mockall::predicate::eq("Diamond"),
            mockall::predicate::eq(10usize),
        )
        .return_once(move |_, _| Ok(entries.clone()))
        .once();

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_leaders_by_clan_ids()
        .return_once(|ids| {
            let mut map = std::collections::HashMap::new();
            if ids.contains(&clan_id_a) {
                map.insert(clan_id_a, leader_a);
            }
            if ids.contains(&clan_id_b) {
                map.insert(clan_id_b, leader_b);
            }
            Ok(map)
        })
        .once();
    mock_clan_repo
        .expect_get_clan_names_by_ids()
        .return_once(|ids| {
            let mut map = std::collections::HashMap::new();
            if ids.contains(&clan_id_a) {
                map.insert(clan_id_a, "Clan A".to_string());
            }
            if ids.contains(&clan_id_b) {
                map.insert(clan_id_b, "Clan B".to_string());
            }
            Ok(map)
        })
        .once();

    let use_case = GetLeaderboardUseCase::new(mock_clan_repo, mock_leaderboard);

    let result = use_case.execute(tier.clone()).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let dto = result.unwrap();
    assert_eq!(dto.tier, tier);
    assert_eq!(dto.entries.len(), 2);
}

// ============================================================
// DeleteClanUseCase Tests
// ============================================================

#[tokio::test]
async fn test_delete_clan_success() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "DeleteMe".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(Some(clan)))
        .once();

    mock_repo
        .expect_delete_clan()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(()))
        .once();

    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();
    mock_leaderboard
        .expect_remove_clan_from_leaderboard()
        .return_once(|_| Ok(()))
        .once();

    let use_case = DeleteClanUseCase::new(mock_repo, mock_leaderboard);
    let dto = DeleteClanDto {
        caller_id: leader_id,
    };

    let result = use_case.execute(clan_id, dto).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
}

#[tokio::test]
async fn test_delete_clan_not_leader() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let other_user = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "DeleteMe".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(Some(clan)))
        .once();

    let mock_leaderboard = MockLeaderboardCacheRepo::new();
    let use_case = DeleteClanUseCase::new(mock_repo, mock_leaderboard);
    let dto = DeleteClanDto {
        caller_id: other_user,
    };

    let result = use_case.execute(clan_id, dto).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, LeagueError::NotLeader(_)));
}

#[tokio::test]
async fn test_delete_clan_not_found() {
    let clan_id = Uuid::new_v4();
    let caller_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(None))
        .once();

    let mock_leaderboard = MockLeaderboardCacheRepo::new();
    let use_case = DeleteClanUseCase::new(mock_repo, mock_leaderboard);
    let dto = DeleteClanDto { caller_id };

    let result = use_case.execute(clan_id, dto).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, LeagueError::ClanNotFound(_)));
}

// ============================================================
// UpdateScoreWithBuffsUseCase Tests
// ============================================================

#[tokio::test]
async fn update_score_with_buffs_bronze_clan() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Bronze Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let members = vec![
        ClanMember::with_joined_at(clan_id, leader_id, MemberRole::Leader, chrono::Utc::now()),
        ClanMember::with_joined_at(clan_id, member_id, MemberRole::Member, chrono::Utc::now()),
    ];

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));
    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(move |_| Ok(members));
    mock_repo.expect_add_score().return_once(|_, _| Ok(()));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_active_buffs()
        .return_once(|_| Ok(vec![]));
    mock_buff_repo
        .expect_get_avg_quiz_score_for_members()
        .return_once(|_| Ok(50));

    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();
    // BronzeSum strategy: 50 + 50 = 100, multiplier 1.0, final_score = 100
    mock_leaderboard
        .expect_update_clan_score()
        .return_once(|_, _| Ok(()));

    let use_case = UpdateScoreWithBuffsUseCase::new(mock_repo, mock_buff_repo, mock_leaderboard);
    let result: Result<ScoreResultDto, AppError> = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.final_score, 100);
    assert_eq!(dto.strategy, "BronzeSum");
}

#[tokio::test]
async fn update_score_with_buffs_clan_not_found() {
    let clan_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo.expect_get_clan_by_id().return_once(|_| Ok(None));

    let mock_buff_repo = MockClanBuffRepo::new();
    let mock_leaderboard = MockLeaderboardCacheRepo::new();

    let use_case = UpdateScoreWithBuffsUseCase::new(mock_repo, mock_buff_repo, mock_leaderboard);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn update_score_with_buffs_applies_multiplier() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Silver Clan".to_string(),
        leader_id,
        ClanTier::Silver,
        500,
        chrono::Utc::now(),
    );

    let members = vec![ClanMember::with_joined_at(
        clan_id,
        leader_id,
        MemberRole::Leader,
        chrono::Utc::now(),
    )];

    let buff =
        ClanBuff::new_productivity_buff(clan_id, chrono::Utc::now() + chrono::Duration::days(7));

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));
    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(move |_| Ok(members));
    mock_repo.expect_add_score().return_once(|_, _| Ok(()));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_active_buffs()
        .return_once(move |_| Ok(vec![buff]));
    mock_buff_repo
        .expect_get_avg_quiz_score_for_members()
        .return_once(|_| Ok(100)); // avg quiz score of 100

    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();
    // SilverWeightedAvg: avg=100, boosted = 100 * 1.1 = 110
    // combined_multiplier = 1.2 (productivity buff), final = 110.0 * 1.2 = 132.0 → 132
    mock_leaderboard
        .expect_update_clan_score()
        .return_once(|_, _| Ok(()));

    let use_case = UpdateScoreWithBuffsUseCase::new(mock_repo, mock_buff_repo, mock_leaderboard);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.final_score, 132);
    assert_eq!(dto.strategy, "SilverWeightedAvg");
    assert!((dto.multiplier - 1.2).abs() < f64::EPSILON);
}

#[tokio::test]
async fn update_score_with_buffs_gold_clan() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let member1 = Uuid::new_v4();
    let member2 = Uuid::new_v4();

    let clan = Clan::with_id(
        clan_id,
        "Gold Clan".to_string(),
        leader_id,
        ClanTier::Gold,
        1200,
        chrono::Utc::now(),
    );

    let members = vec![
        ClanMember::with_joined_at(clan_id, leader_id, MemberRole::Leader, chrono::Utc::now()),
        ClanMember::with_joined_at(clan_id, member1, MemberRole::Member, chrono::Utc::now()),
        ClanMember::with_joined_at(clan_id, member2, MemberRole::Member, chrono::Utc::now()),
    ];

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));
    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(move |_| Ok(members));
    mock_repo.expect_add_score().return_once(|_, _| Ok(()));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_active_buffs()
        .return_once(|_| Ok(vec![]));
    mock_buff_repo
        .expect_get_avg_quiz_score_for_members()
        .return_once(|_| Ok(200));

    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();
    // GoldWeightedAvg: avg=200, with attendance = 200 * 1.2 = 240
    // combined_multiplier = 1.0, final = 240
    mock_leaderboard
        .expect_update_clan_score()
        .return_once(|_, _| Ok(()));

    let use_case = UpdateScoreWithBuffsUseCase::new(mock_repo, mock_buff_repo, mock_leaderboard);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.final_score, 240);
    assert_eq!(dto.strategy, "GoldWeightedAvg");
}

// ============================================================
// ProcessBuffsUseCase Tests
// ============================================================

#[tokio::test]
async fn process_buffs_activates_productivity_buff() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let members = vec![ClanMember::with_joined_at(
        clan_id,
        leader_id,
        MemberRole::Leader,
        chrono::Utc::now(),
    )];

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(move |_| Ok(members));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_avg_accuracy_for_members()
        .return_once(|_| Ok(0.7));
    mock_buff_repo
        .expect_get_mission_completion_rate_for_clan()
        .return_once(|_| Ok(0.6));
    mock_buff_repo
        .expect_get_buff_by_name()
        .returning(|_, _| Ok(None));
    mock_buff_repo
        .expect_activate_buff()
        .return_once(|_| Ok(()));

    let use_case = ProcessBuffsUseCase::new(mock_repo, mock_buff_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto: BuffProcessResultDto = result.unwrap();
    assert!(dto.buffs_added.contains(&"Productivity Buff".to_string()));
    assert!(dto.buffs_removed.is_empty());
}

#[tokio::test]
async fn process_buffs_activates_low_accuracy_debuff() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let members = vec![ClanMember::with_joined_at(
        clan_id,
        leader_id,
        MemberRole::Leader,
        chrono::Utc::now(),
    )];

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(move |_| Ok(members));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_avg_accuracy_for_members()
        .return_once(|_| Ok(0.3));
    mock_buff_repo
        .expect_get_mission_completion_rate_for_clan()
        .return_once(|_| Ok(0.2));
    mock_buff_repo
        .expect_get_buff_by_name()
        .returning(|_, _| Ok(None));
    mock_buff_repo
        .expect_activate_buff()
        .return_once(|_| Ok(()));

    let use_case = ProcessBuffsUseCase::new(mock_repo, mock_buff_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert!(
        dto.buffs_added
            .contains(&"Low Accuracy Penalty".to_string())
    );
    assert!(dto.buffs_removed.is_empty());
}

#[tokio::test]
async fn process_buffs_deactivates_productivity_buff_when_rate_low() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    let members = vec![ClanMember::with_joined_at(
        clan_id,
        leader_id,
        MemberRole::Leader,
        chrono::Utc::now(),
    )];

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(move |_| Ok(members));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_avg_accuracy_for_members()
        .return_once(|_| Ok(0.8));
    mock_buff_repo
        .expect_get_mission_completion_rate_for_clan()
        .return_once(|_| Ok(0.3));
    mock_buff_repo
        .expect_get_buff_by_name()
        .returning(move |_, name| {
            if name == "Productivity Buff" {
                Ok(Some(ClanBuff::new_productivity_buff(clan_id, expires_at)))
            } else {
                Ok(None)
            }
        });
    mock_buff_repo
        .expect_deactivate_buff_by_name()
        .return_once(|_, _| Ok(()));

    let use_case = ProcessBuffsUseCase::new(mock_repo, mock_buff_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert!(dto.buffs_removed.contains(&"Productivity Buff".to_string()));
    assert!(dto.buffs_added.is_empty());
}

#[tokio::test]
async fn process_buffs_deactivates_accuracy_debuff_when_accuracy_high() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    let members = vec![ClanMember::with_joined_at(
        clan_id,
        leader_id,
        MemberRole::Leader,
        chrono::Utc::now(),
    )];

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(move |_| Ok(members));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_avg_accuracy_for_members()
        .return_once(|_| Ok(0.7));
    mock_buff_repo
        .expect_get_mission_completion_rate_for_clan()
        .return_once(|_| Ok(0.6));
    mock_buff_repo
        .expect_get_buff_by_name()
        .returning(move |_, name| {
            if name == "Low Accuracy Penalty" {
                Ok(Some(ClanBuff::new_low_accuracy_debuff(clan_id, expires_at)))
            } else {
                Ok(None)
            }
        });
    mock_buff_repo
        .expect_deactivate_buff_by_name()
        .return_once(|_, _| Ok(()));
    mock_buff_repo
        .expect_activate_buff()
        .return_once(|_| Ok(()));

    let use_case = ProcessBuffsUseCase::new(mock_repo, mock_buff_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert!(
        dto.buffs_removed
            .contains(&"Low Accuracy Penalty".to_string())
    );
    assert!(dto.buffs_added.contains(&"Productivity Buff".to_string()));
}

#[tokio::test]
async fn process_buffs_both_buffs_active_simultaneously() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

    let members = vec![ClanMember::with_joined_at(
        clan_id,
        leader_id,
        MemberRole::Leader,
        chrono::Utc::now(),
    )];

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_get_members_by_clan_id()
        .return_once(move |_| Ok(members));

    let mut mock_buff_repo = MockClanBuffRepo::new();
    mock_buff_repo
        .expect_get_avg_accuracy_for_members()
        .return_once(|_| Ok(0.3));
    mock_buff_repo
        .expect_get_mission_completion_rate_for_clan()
        .return_once(|_| Ok(0.8));
    mock_buff_repo
        .expect_get_buff_by_name()
        .returning(|_, _| Ok(None));
    mock_buff_repo.expect_activate_buff().returning(|_| Ok(()));

    let use_case = ProcessBuffsUseCase::new(mock_repo, mock_buff_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert!(
        dto.buffs_added
            .contains(&"Low Accuracy Penalty".to_string())
    );
    assert!(dto.buffs_added.contains(&"Productivity Buff".to_string()));
    assert!(dto.buffs_removed.is_empty());
}

// ============================================================
// TriggerSeasonEndUseCase Tests
// ============================================================

#[tokio::test]
async fn trigger_season_end_promotes_and_demotes() {
    let season_id = Uuid::new_v4();

    let season = Season::with_id(
        season_id,
        "Silver Season".to_string(),
        ClanTier::Silver,
        chrono::Utc::now() - chrono::Duration::days(30),
        chrono::Utc::now() + chrono::Duration::days(1),
        true,
    );

    let clan_a_id = Uuid::new_v4();
    let clan_b_id = Uuid::new_v4();
    let clan_c_id = Uuid::new_v4();
    let clan_x_id = Uuid::new_v4();
    let clan_y_id = Uuid::new_v4();
    let clan_z_id = Uuid::new_v4();

    let results: Vec<(Uuid, String, i64, i64)> = vec![
        (clan_a_id, "Clan A".to_string(), 1, 1000),
        (clan_b_id, "Clan B".to_string(), 2, 900),
        (clan_c_id, "Clan C".to_string(), 3, 800),
        (clan_x_id, "Clan X".to_string(), 4, 200),
        (clan_y_id, "Clan Y".to_string(), 5, 150),
        (clan_z_id, "Clan Z".to_string(), 6, 100),
    ];

    let mut mock_repo = MockSeasonRepo::new();
    mock_repo
        .expect_get_season_by_id()
        .return_once(move |_| Ok(Some(season)));
    mock_repo
        .expect_get_season_results()
        .return_once(move |_, _| Ok(results));
    // 3 promotions to Gold + 3 demotions to Bronze = 6 update_clan_tier calls
    mock_repo.expect_update_clan_tier().returning(|_, _| Ok(()));
    mock_repo.expect_mark_season_ended().return_once(|_| Ok(()));

    let use_case = TriggerSeasonEndUseCase::new(mock_repo);
    let result: Result<SeasonResultDto, AppError> = use_case.execute(season_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.promoted.len(), 3);
    assert_eq!(dto.demoted.len(), 3);
    assert_eq!(dto.season_id, season_id);
}

#[tokio::test]
async fn trigger_season_end_season_not_found() {
    let season_id = Uuid::new_v4();

    let mut mock_repo = MockSeasonRepo::new();
    mock_repo
        .expect_get_season_by_id()
        .return_once(|_| Ok(None));

    let use_case = TriggerSeasonEndUseCase::new(mock_repo);
    let result: Result<SeasonResultDto, AppError> = use_case.execute(season_id).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn trigger_season_end_bronze_no_demotion() {
    let season_id = Uuid::new_v4();

    // Bronze tier: can promote to Silver but cannot demote (no tier below Bronze)
    let season = Season::with_id(
        season_id,
        "Bronze Season".to_string(),
        ClanTier::Bronze,
        chrono::Utc::now() - chrono::Duration::days(30),
        chrono::Utc::now() + chrono::Duration::days(1),
        true,
    );

    let clan_a_id = Uuid::new_v4();
    let clan_b_id = Uuid::new_v4();
    let clan_c_id = Uuid::new_v4();
    let clan_x_id = Uuid::new_v4();
    let clan_y_id = Uuid::new_v4();
    let clan_z_id = Uuid::new_v4();

    let results: Vec<(Uuid, String, i64, i64)> = vec![
        (clan_a_id, "Clan A".to_string(), 1, 500),
        (clan_b_id, "Clan B".to_string(), 2, 400),
        (clan_c_id, "Clan C".to_string(), 3, 300),
        (clan_x_id, "Clan X".to_string(), 4, 100),
        (clan_y_id, "Clan Y".to_string(), 5, 50),
        (clan_z_id, "Clan Z".to_string(), 6, 10),
    ];

    let mut mock_repo = MockSeasonRepo::new();
    mock_repo
        .expect_get_season_by_id()
        .return_once(move |_| Ok(Some(season)));
    mock_repo
        .expect_get_season_results()
        .return_once(move |_, _| Ok(results));
    // Only promotions (3 to Silver), no demotions (no tier below Bronze)
    mock_repo.expect_update_clan_tier().returning(|_, _| Ok(()));
    mock_repo.expect_mark_season_ended().return_once(|_| Ok(()));

    let use_case = TriggerSeasonEndUseCase::new(mock_repo);
    let result: Result<SeasonResultDto, AppError> = use_case.execute(season_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.promoted.len(), 3);
    assert_eq!(dto.demoted.len(), 0);
}

#[tokio::test]
async fn trigger_season_end_diamond_no_promotion() {
    let season_id = Uuid::new_v4();

    // Diamond tier: can demote to Gold but cannot promote (no tier above Diamond)
    let season = Season::with_id(
        season_id,
        "Diamond Season".to_string(),
        ClanTier::Diamond,
        chrono::Utc::now() - chrono::Duration::days(30),
        chrono::Utc::now() + chrono::Duration::days(1),
        true,
    );

    let clan_a_id = Uuid::new_v4();
    let clan_b_id = Uuid::new_v4();
    let clan_c_id = Uuid::new_v4();
    let clan_x_id = Uuid::new_v4();
    let clan_y_id = Uuid::new_v4();
    let clan_z_id = Uuid::new_v4();

    let results: Vec<(Uuid, String, i64, i64)> = vec![
        (clan_a_id, "Clan A".to_string(), 1, 5000),
        (clan_b_id, "Clan B".to_string(), 2, 4000),
        (clan_c_id, "Clan C".to_string(), 3, 3000),
        (clan_x_id, "Clan X".to_string(), 4, 200),
        (clan_y_id, "Clan Y".to_string(), 5, 100),
        (clan_z_id, "Clan Z".to_string(), 6, 50),
    ];

    let mut mock_repo = MockSeasonRepo::new();
    mock_repo
        .expect_get_season_by_id()
        .return_once(move |_| Ok(Some(season)));
    mock_repo
        .expect_get_season_results()
        .return_once(move |_, _| Ok(results));
    // No promotions (Diamond is top), 3 demotions to Gold
    mock_repo.expect_update_clan_tier().returning(|_, _| Ok(()));
    mock_repo.expect_mark_season_ended().return_once(|_| Ok(()));

    let use_case = TriggerSeasonEndUseCase::new(mock_repo);
    let result: Result<SeasonResultDto, AppError> = use_case.execute(season_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.promoted.len(), 0);
    assert_eq!(dto.demoted.len(), 3);
}

// ============================================================
// GetClanBuffsUseCase Tests
// ============================================================

#[tokio::test]
async fn get_clan_buffs_returns_buffs_and_debuffs() {
    let clan_id = Uuid::new_v4();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    let buff = ClanBuff::new_productivity_buff(clan_id, expires_at);
    let debuff = ClanBuff::new_low_accuracy_debuff(clan_id, expires_at);

    let mut mock_repo = MockClanBuffRepo::new();
    mock_repo
        .expect_get_active_buffs()
        .return_once(move |_| Ok(vec![buff, debuff]));

    let use_case = GetClanBuffsUseCase::new(mock_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto: ClanBuffsDto = result.unwrap();
    assert_eq!(dto.buffs.len(), 1);
    assert_eq!(dto.debuffs.len(), 1);
    assert_eq!(dto.buffs[0].name, "Productivity Buff");
    assert_eq!(dto.debuffs[0].name, "Low Accuracy Penalty");
}

#[tokio::test]
async fn get_clan_buffs_empty_when_no_active_buffs() {
    let clan_id = Uuid::new_v4();

    let mut mock_repo = MockClanBuffRepo::new();
    mock_repo
        .expect_get_active_buffs()
        .return_once(|_| Ok(vec![]));

    let use_case = GetClanBuffsUseCase::new(mock_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert!(dto.buffs.is_empty());
    assert!(dto.debuffs.is_empty());
}

#[tokio::test]
async fn get_clan_buffs_repo_error() {
    let clan_id = Uuid::new_v4();

    let mut mock_repo = MockClanBuffRepo::new();
    mock_repo
        .expect_get_active_buffs()
        .return_once(|_| Err(AppError::InternalServer("DB error".to_string())));

    let use_case = GetClanBuffsUseCase::new(mock_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn get_clan_buffs_only_buffs_no_debuffs() {
    let clan_id = Uuid::new_v4();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    let buff = ClanBuff::new_productivity_buff(clan_id, expires_at);

    let mut mock_repo = MockClanBuffRepo::new();
    mock_repo
        .expect_get_active_buffs()
        .return_once(move |_| Ok(vec![buff]));

    let use_case = GetClanBuffsUseCase::new(mock_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.buffs.len(), 1);
    assert!(dto.debuffs.is_empty());
    assert_eq!(dto.buffs[0].name, "Productivity Buff");
    assert!((dto.buffs[0].multiplier - 1.2).abs() < f64::EPSILON);
}

#[tokio::test]
async fn get_clan_buffs_only_debuffs_no_buffs() {
    let clan_id = Uuid::new_v4();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    let debuff = ClanBuff::new_low_accuracy_debuff(clan_id, expires_at);

    let mut mock_repo = MockClanBuffRepo::new();
    mock_repo
        .expect_get_active_buffs()
        .return_once(move |_| Ok(vec![debuff]));

    let use_case = GetClanBuffsUseCase::new(mock_repo);
    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert!(dto.buffs.is_empty());
    assert_eq!(dto.debuffs.len(), 1);
    assert_eq!(dto.debuffs[0].name, "Low Accuracy Penalty");
    assert!((dto.debuffs[0].multiplier - 0.8).abs() < f64::EPSILON);
}

// ============================================================
// CreateJoinRequestUseCase Tests
// ============================================================

#[tokio::test]
async fn create_join_request_success() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));
    mock_clan_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_pending_request_by_user()
        .return_once(|_, _| Ok(None));
    mock_join_repo
        .expect_create_request()
        .return_once(|_| Ok(()));

    let use_case = CreateJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = CreateJoinRequestDto { clan_id, user_id };
    let result = use_case.execute(dto).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let response = result.unwrap();
    assert_eq!(response.status, "Pending");
    assert_eq!(response.clan_id, clan_id);
    assert_eq!(response.user_id, user_id);
}

#[tokio::test]
async fn create_join_request_clan_not_found() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(None));

    let mock_join_repo = MockClanJoinRequestRepo::new();
    let use_case = CreateJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = CreateJoinRequestDto { clan_id, user_id };
    let result = use_case.execute(dto).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeagueError::ClanNotFound(_)));
}

#[tokio::test]
async fn create_join_request_user_already_in_clan() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));
    mock_clan_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(true));

    let mock_join_repo = MockClanJoinRequestRepo::new();
    let use_case = CreateJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = CreateJoinRequestDto { clan_id, user_id };
    let result = use_case.execute(dto).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LeagueError::UserAlreadyInClan(_)
    ));
}

#[tokio::test]
async fn create_join_request_duplicate() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );
    let existing_request = ClanJoinRequest::new(clan_id, user_id);

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));
    mock_clan_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_pending_request_by_user()
        .return_once(|_, _| Ok(Some(existing_request)));

    let use_case = CreateJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = CreateJoinRequestDto { clan_id, user_id };
    let result = use_case.execute(dto).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LeagueError::DuplicateRequest(_)
    ));
}

// ============================================================
// ApproveJoinRequestUseCase Tests
// ============================================================

#[tokio::test]
async fn approve_join_request_success() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );
    let mut request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Pending,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    request.approve();

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(Some(clan)));
    mock_clan_repo
        .expect_is_user_in_any_clan()
        .with(mockall::predicate::eq(user_id))
        .return_once(|_| Ok(false));
    mock_clan_repo.expect_add_member().return_once(|_| Ok(()));

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    {
        let req = ClanJoinRequest::with_id(
            request_id,
            clan_id,
            user_id,
            RequestStatus::Pending,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );
        mock_join_repo
            .expect_get_request_by_id()
            .with(mockall::predicate::eq(request_id))
            .return_once(|_| Ok(Some(req)));
    }
    mock_join_repo
        .expect_update_request_status()
        .return_once(|_, _| Ok(()));

    let use_case = ApproveJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: leader_id,
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let response = result.unwrap();
    assert_eq!(response.status, "Approved");
}

#[tokio::test]
async fn approve_join_request_not_leader() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let other_user = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Pending,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(Some(clan)));

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));

    let use_case = ApproveJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: other_user,
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeagueError::NotLeader(_)));
}

#[tokio::test]
async fn approve_join_request_not_found() {
    let request_id = Uuid::new_v4();
    let caller_id = Uuid::new_v4();

    let mock_clan_repo = MockClanRepositoryRepo::new();
    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(None));

    let use_case = ApproveJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto { caller_id };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LeagueError::RequestNotFound(_)
    ));
}

#[tokio::test]
async fn approve_join_request_already_processed() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let caller_id = Uuid::new_v4();
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Approved,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mock_clan_repo = MockClanRepositoryRepo::new();
    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));

    let use_case = ApproveJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto { caller_id };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LeagueError::RequestAlreadyProcessed(_)
    ));
}

// ============================================================
// RejectJoinRequestUseCase Tests
// ============================================================

#[tokio::test]
async fn reject_join_request_success() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Pending,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(Some(clan)));

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));
    mock_join_repo
        .expect_update_request_status()
        .return_once(|_, _| Ok(()));

    let use_case = RejectJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: leader_id,
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let response = result.unwrap();
    assert_eq!(response.status, "Rejected");
}

#[tokio::test]
async fn reject_join_request_not_leader() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let other_user = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Pending,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(Some(clan)));

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));

    let use_case = RejectJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: other_user,
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeagueError::NotLeader(_)));
}

// ============================================================
// GetPendingRequestsUseCase Tests
// ============================================================

#[tokio::test]
async fn get_pending_requests_success() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let requests = vec![
        ClanJoinRequest::new(clan_id, user1),
        ClanJoinRequest::new(clan_id, user2),
    ];

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_pending_requests_by_clan()
        .return_once(|_| Ok(requests));

    let use_case = GetPendingRequestsUseCase::new(mock_clan_repo, mock_join_repo);
    let result = use_case.execute(clan_id, leader_id).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let dtos = result.unwrap();
    assert_eq!(dtos.len(), 2);
}

#[tokio::test]
async fn get_pending_requests_not_leader() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let other_user = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));

    let mock_join_repo = MockClanJoinRequestRepo::new();
    let use_case = GetPendingRequestsUseCase::new(mock_clan_repo, mock_join_repo);
    let result = use_case.execute(clan_id, other_user).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeagueError::NotLeader(_)));
}

#[tokio::test]
async fn get_pending_requests_empty() {
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan)));

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_pending_requests_by_clan()
        .return_once(|_| Ok(vec![]));

    let use_case = GetPendingRequestsUseCase::new(mock_clan_repo, mock_join_repo);
    let result = use_case.execute(clan_id, leader_id).await;

    assert!(result.is_ok());
    let dtos = result.unwrap();
    assert!(dtos.is_empty());
}

// ============================================================
// Edge Cases — Join Request State Transitions & Race Conditions
// ============================================================

// Approve an already-rejected request -> should be rejected as already processed
#[tokio::test]
async fn approve_already_rejected_request_fails() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Rejected,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mock_clan_repo = MockClanRepositoryRepo::new();
    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));

    let use_case = ApproveJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: Uuid::new_v4(),
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LeagueError::RequestAlreadyProcessed(_)
    ));
}

// Reject an already-approved request -> should be rejected as already processed
#[tokio::test]
async fn reject_already_approved_request_fails() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Approved,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mock_clan_repo = MockClanRepositoryRepo::new();
    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));

    let use_case = RejectJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: Uuid::new_v4(),
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LeagueError::RequestAlreadyProcessed(_)
    ));
}

// Approve when user joined another clan in the meantime -> auto-reject + error
#[tokio::test]
async fn approve_user_joined_different_clan_in_meantime() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Pending,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(Some(clan)));
    mock_clan_repo
        .expect_is_user_in_any_clan()
        .with(mockall::predicate::eq(user_id))
        .return_once(|_| Ok(true));
    mock_clan_repo.expect_add_member().never();

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));
    mock_join_repo
        .expect_update_request_status()
        .return_once(|_, _| Ok(()));

    let use_case = ApproveJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: leader_id,
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LeagueError::UserAlreadyInClan(_)
    ));
}

// Approve when the clan was deleted between request creation and approval -> ClanNotFound
#[tokio::test]
async fn approve_clan_deleted_before_approval() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Pending,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(None)); // Clan no longer exists

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));

    let use_case = ApproveJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: Uuid::new_v4(),
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeagueError::ClanNotFound(_)));
}

// Reject non-existent request -> RequestNotFound
#[tokio::test]
async fn reject_non_existent_request_fails() {
    let request_id = Uuid::new_v4();

    let mock_clan_repo = MockClanRepositoryRepo::new();
    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(None));

    let use_case = RejectJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: Uuid::new_v4(),
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LeagueError::RequestNotFound(_)
    ));
}

// Reject when the clan was deleted between request creation and rejection -> ClanNotFound
#[tokio::test]
async fn reject_clan_deleted_before_rejection() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Pending,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(None)); // Clan no longer exists

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));

    let use_case = RejectJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: Uuid::new_v4(),
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeagueError::ClanNotFound(_)));
}

// Request to non-existent clan -> ClanNotFound
#[tokio::test]
async fn create_join_request_for_deleted_clan() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(None));

    let mock_join_repo = MockClanJoinRequestRepo::new();
    let use_case = CreateJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = CreateJoinRequestDto { clan_id, user_id };
    let result = use_case.execute(dto).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeagueError::ClanNotFound(_)));
}

// User can have pending requests to multiple different clans simultaneously
#[tokio::test]
async fn create_join_request_to_different_clans_allowed() {
    let clan_a_id = Uuid::new_v4();
    let _clan_b_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let clan_a = Clan::with_id(
        clan_a_id,
        "Clan A".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(Some(clan_a)));
    mock_clan_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    // No pending request for (user_id, clan_a_id) pair
    mock_join_repo
        .expect_get_pending_request_by_user()
        .with(
            mockall::predicate::eq(user_id),
            mockall::predicate::eq(clan_a_id),
        )
        .return_once(|_, _| Ok(None));
    mock_join_repo
        .expect_create_request()
        .return_once(|_| Ok(()));

    let use_case = CreateJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = CreateJoinRequestDto {
        clan_id: clan_a_id,
        user_id,
    };
    let result = use_case.execute(dto).await;

    assert!(result.is_ok(), "Should allow request to different clan");
    assert_eq!(result.unwrap().clan_id, clan_a_id);
}

// Get pending requests for a non-existent clan -> ClanNotFound
#[tokio::test]
async fn get_pending_requests_for_non_existent_clan() {
    let clan_id = Uuid::new_v4();

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .return_once(|_| Ok(None));

    let mock_join_repo = MockClanJoinRequestRepo::new();
    let use_case = GetPendingRequestsUseCase::new(mock_clan_repo, mock_join_repo);
    let result = use_case.execute(clan_id, Uuid::new_v4()).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LeagueError::ClanNotFound(_)));
}

// Approve where the add_member succeeds but update_request_status fails
// The request is logically approved but the status write fails -> error propagated
#[tokio::test]
async fn approve_add_member_fails_after_status_update() {
    let request_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();
    let clan = Clan::with_id(
        clan_id,
        "Test Clan".to_string(),
        leader_id,
        ClanTier::Bronze,
        0,
        chrono::Utc::now(),
    );
    let request = ClanJoinRequest::with_id(
        request_id,
        clan_id,
        user_id,
        RequestStatus::Pending,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let mut mock_clan_repo = MockClanRepositoryRepo::new();
    mock_clan_repo
        .expect_get_clan_by_id()
        .with(mockall::predicate::eq(clan_id))
        .return_once(|_| Ok(Some(clan)));
    mock_clan_repo
        .expect_is_user_in_any_clan()
        .with(mockall::predicate::eq(user_id))
        .return_once(|_| Ok(false));
    mock_clan_repo.expect_add_member().return_once(|_| {
        Err(AppError::InternalServer(
            "FK violation: user not found".to_string(),
        ))
    });

    let mut mock_join_repo = MockClanJoinRequestRepo::new();
    mock_join_repo
        .expect_get_request_by_id()
        .with(mockall::predicate::eq(request_id))
        .return_once(|_| Ok(Some(request)));
    // Status was already updated to Approved
    mock_join_repo
        .expect_update_request_status()
        .return_once(|_, _| Ok(()));

    let use_case = ApproveJoinRequestUseCase::new(mock_clan_repo, mock_join_repo);
    let dto = ApproveRejectDto {
        caller_id: leader_id,
    };
    let result = use_case.execute(request_id, dto).await;

    assert!(result.is_err());
}

// ============================================================
// Old Edge Cases
// ============================================================

#[tokio::test]
async fn invalid_uuid_handling() {
    let random_uuid = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo.expect_get_clan_by_id().return_once(|_| Ok(None));

    let use_case = GetClanDetailUseCase::new(mock_repo, MockClanBuffRepo::new());

    let result = use_case.execute(random_uuid).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn empty_database_get_clan() {
    let random_uuid = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo.expect_get_clan_by_id().return_once(|_| Ok(None));

    let use_case = GetClanDetailUseCase::new(mock_repo, MockClanBuffRepo::new());

    let result = use_case.execute(random_uuid).await;

    assert!(result.is_err());
}
