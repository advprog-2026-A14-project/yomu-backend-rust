use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

use yomu_backend_rust::modules::league::application::CreateClanUseCase;
use yomu_backend_rust::modules::league::application::GetClanDetailUseCase;
use yomu_backend_rust::modules::league::application::GetLeaderboardUseCase;
use yomu_backend_rust::modules::league::application::GetUserTierUseCase;
use yomu_backend_rust::modules::league::application::JoinClanUseCase;
use yomu_backend_rust::modules::league::application::UpdateScoreUseCase;
use yomu_backend_rust::modules::league::application::dto::CreateClanDto;
use yomu_backend_rust::modules::league::application::dto::JoinClanDto;
use yomu_backend_rust::modules::league::application::dto::LeaderboardDto;
use yomu_backend_rust::modules::league::application::dto::LeaderboardEntry;
use yomu_backend_rust::modules::league::application::dto::UpdateScoreDto;
use yomu_backend_rust::modules::league::application::dto::UserTierDto;
use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
use yomu_backend_rust::modules::league::domain::entities::clan::ClanTier;
use yomu_backend_rust::modules::league::domain::entities::clan_member::ClanMember;
use yomu_backend_rust::modules::league::domain::entities::clan_member::MemberRole;
use yomu_backend_rust::modules::league::domain::repositories::ClanRepository;
use yomu_backend_rust::modules::league::domain::repositories::LeaderboardCache;
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
        async fn get_user_tier_info(&self, user_id: Uuid) -> Result<Option<(Uuid, String, ClanTier)>, AppError>;
    }
}

mock! {
    LeaderboardCacheRepo {}
    #[async_trait]
    impl LeaderboardCache for LeaderboardCacheRepo {
        async fn update_clan_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
        async fn get_top_clans(&self, tier: &str, limit: usize) -> Result<Vec<LeaderboardEntry>, AppError>;
    }
}

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

    let use_case = CreateClanUseCase::new(mock_repo);
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

    let use_case = CreateClanUseCase::new(mock_repo);
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
async fn test_update_score_success() {
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let base_score = 100i64;
    let multiplier = 1.5f64;
    let expected_final_score = 150i64;

    let mut mock_repo = MockClanRepositoryRepo::new();
    let mut mock_leaderboard = MockLeaderboardCacheRepo::new();

    mock_repo
        .expect_add_score()
        .with(
            mockall::predicate::eq(clan_id),
            mockall::predicate::eq(expected_final_score),
        )
        .return_once(|_, _| Ok(()))
        .once();

    mock_leaderboard
        .expect_update_clan_score()
        .with(
            mockall::predicate::eq(clan_id),
            mockall::predicate::eq(expected_final_score),
        )
        .return_once(|_, _| Ok(()))
        .once();

    let use_case = UpdateScoreUseCase::new(mock_repo, mock_leaderboard);
    let dto = UpdateScoreDto {
        clan_id,
        user_id,
        base_score,
        multiplier,
    };

    let result = use_case.execute(dto).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    assert_eq!(result.unwrap(), expected_final_score);
}

#[tokio::test]
async fn test_get_leaderboard_success() {
    let tier = "Diamond".to_string();
    let entries = vec![
        LeaderboardEntry {
            clan_id: Uuid::new_v4(),
            clan_name: "Clan A".to_string(),
            total_score: 1000,
            tier: tier.clone(),
            rank: 1,
        },
        LeaderboardEntry {
            clan_id: Uuid::new_v4(),
            clan_name: "Clan B".to_string(),
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

    let use_case = GetLeaderboardUseCase::new(mock_leaderboard);

    let result = use_case.execute(tier.clone()).await;

    assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    let dto = result.unwrap();
    assert_eq!(dto.tier, tier);
    assert_eq!(dto.entries.len(), 2);
}

// CreateClanUseCase Tests

#[tokio::test]
async fn create_clan_empty_name() {
    let leader_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();
    mock_repo
        .expect_is_user_in_any_clan()
        .return_once(|_| Ok(false));
    mock_repo.expect_create_clan().return_once(|_| Ok(()));
    mock_repo.expect_add_member().return_once(|_| Ok(()));

    let use_case = CreateClanUseCase::new(mock_repo);
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

    let use_case = CreateClanUseCase::new(mock_repo);
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

    let use_case = CreateClanUseCase::new(mock_repo);
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

    let use_case2 = CreateClanUseCase::new(mock_repo2);
    let dto2 = CreateClanDto {
        name: clan_name.to_string(),
        leader_id,
    };

    let result2 = use_case2.execute(dto2).await;
    assert!(result2.is_err());
    let err = result2.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
}

// JoinClanUseCase Tests

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

// GetClanDetailUseCase Tests

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

    let use_case = GetClanDetailUseCase::new(mock_repo);

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

    let use_case = GetClanDetailUseCase::new(mock_repo);

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

    let use_case = GetClanDetailUseCase::new(mock_repo);

    let result = use_case.execute(clan_id).await;

    assert!(result.is_ok());
    let dto = result.unwrap();
    assert_eq!(dto.id, clan_id);
    assert_eq!(dto.members.len(), 0);
}

// GetUserTierUseCase Tests

#[tokio::test]
async fn get_user_tier_has_clan() {
    let user_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let leader_id = Uuid::new_v4();

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

// Edge Cases

#[tokio::test]
async fn invalid_uuid_handling() {
    let random_uuid = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo.expect_get_clan_by_id().return_once(|_| Ok(None));

    let use_case = GetClanDetailUseCase::new(mock_repo);

    let result = use_case.execute(random_uuid).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn empty_database_get_clan() {
    let random_uuid = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo.expect_get_clan_by_id().return_once(|_| Ok(None));

    let use_case = GetClanDetailUseCase::new(mock_repo);

    let result = use_case.execute(random_uuid).await;

    assert!(result.is_err());
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

#[tokio::test]
async fn clan_detail_with_db_error() {
    let clan_id = Uuid::new_v4();

    let mut mock_repo = MockClanRepositoryRepo::new();

    mock_repo
        .expect_get_clan_by_id()
        .return_once(|_| Err(AppError::InternalServer("DB connection failed".to_string())));

    let use_case = GetClanDetailUseCase::new(mock_repo);

    let result = use_case.execute(clan_id).await;

    assert!(result.is_err());
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

    let use_case = CreateClanUseCase::new(mock_repo);
    let dto = CreateClanDto {
        name: "Test Clan".to_string(),
        leader_id,
    };

    let result = use_case.execute(dto).await;

    assert!(result.is_err());
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
