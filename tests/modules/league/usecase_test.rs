use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

#[async_trait]
pub trait ClanRepository: Send + Sync {
    async fn create_clan(&self, clan: &Clan) -> Result<(), AppError>;
    async fn get_clan_by_id(&self, clan_id: Uuid) -> Result<Option<Clan>, AppError>;
    async fn add_member(&self, member: &ClanMember) -> Result<(), AppError>;
    async fn is_user_in_any_clan(&self, user_id: Uuid) -> Result<bool, AppError>;
    async fn get_user_clan_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;
    async fn add_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
}

#[async_trait]
pub trait LeaderboardCache: Send + Sync {
    async fn update_clan_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
    async fn get_top_clans(
        &self,
        tier: &str,
        limit: usize,
    ) -> Result<Vec<LeaderboardEntry>, AppError>;
}

use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
use yomu_backend_rust::modules::league::domain::entities::clan::ClanTier;
use yomu_backend_rust::modules::league::domain::entities::clan_member::ClanMember;
use yomu_backend_rust::modules::league::domain::entities::clan_member::MemberRole;
use yomu_backend_rust::shared::domain::base_error::AppError;

#[derive(Debug, Clone)]
pub struct CreateClanDto {
    pub name: String,
    pub leader_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct JoinClanDto {
    pub clan_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateScoreDto {
    pub clan_id: Uuid,
    pub user_id: Uuid,
    pub base_score: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub clan_id: Uuid,
    pub clan_name: String,
    pub total_score: i64,
    pub tier: String,
    pub rank: usize,
}

#[derive(Debug, Clone)]
pub struct LeaderboardDto {
    pub entries: Vec<LeaderboardEntry>,
    pub tier: String,
}

pub struct CreateClanUseCase<R: ClanRepository> {
    repo: R,
}

impl<R: ClanRepository> CreateClanUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, dto: CreateClanDto) -> Result<Clan, AppError> {
        if self.repo.is_user_in_any_clan(dto.leader_id).await? {
            return Err(AppError::BadRequest(
                "User is already in a clan".to_string(),
            ));
        }

        let clan = Clan::new(dto.name, dto.leader_id);
        self.repo.create_clan(&clan).await?;

        let member = ClanMember::new(clan.id(), dto.leader_id, MemberRole::Leader);
        self.repo.add_member(&member).await?;

        Ok(clan)
    }
}

pub struct JoinClanUseCase<R: ClanRepository> {
    repo: R,
}

impl<R: ClanRepository> JoinClanUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, dto: JoinClanDto) -> Result<ClanMember, AppError> {
        let clan = self.repo.get_clan_by_id(dto.clan_id).await?;
        if clan.is_none() {
            return Err(AppError::NotFound("Clan not found".to_string()));
        }

        if self.repo.is_user_in_any_clan(dto.user_id).await? {
            return Err(AppError::BadRequest(
                "User is already in a clan".to_string(),
            ));
        }

        let member = ClanMember::new(dto.clan_id, dto.user_id, MemberRole::Member);
        self.repo.add_member(&member).await?;

        Ok(member)
    }
}

pub struct UpdateScoreUseCase<R: ClanRepository, L: LeaderboardCache> {
    repo: R,
    leaderboard: L,
}

impl<R: ClanRepository, L: LeaderboardCache> UpdateScoreUseCase<R, L> {
    pub fn new(repo: R, leaderboard: L) -> Self {
        Self { repo, leaderboard }
    }

    pub async fn execute(&self, dto: UpdateScoreDto) -> Result<i64, AppError> {
        let final_score = (dto.base_score as f64 * dto.multiplier) as i64;

        self.repo.add_score(dto.clan_id, final_score).await?;

        self.leaderboard
            .update_clan_score(dto.clan_id, final_score)
            .await?;

        Ok(final_score)
    }
}

pub struct GetLeaderboardUseCase<L: LeaderboardCache> {
    leaderboard: L,
}

impl<L: LeaderboardCache> GetLeaderboardUseCase<L> {
    pub fn new(leaderboard: L) -> Self {
        Self { leaderboard }
    }

    pub async fn execute(&self, tier: String) -> Result<LeaderboardDto, AppError> {
        let entries = self.leaderboard.get_top_clans(&tier, 10).await?;

        Ok(LeaderboardDto { entries, tier })
    }
}

mock! {
    ClanRepositoryRepo {}
    #[async_trait]
    impl ClanRepository for ClanRepositoryRepo {
        async fn create_clan(&self, clan: &Clan) -> Result<(), AppError>;
        async fn get_clan_by_id(&self, clan_id: Uuid) -> Result<Option<Clan>, AppError>;
        async fn add_member(&self, member: &ClanMember) -> Result<(), AppError>;
        async fn is_user_in_any_clan(&self, user_id: Uuid) -> Result<bool, AppError>;
        async fn get_user_clan_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;
        async fn add_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
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
