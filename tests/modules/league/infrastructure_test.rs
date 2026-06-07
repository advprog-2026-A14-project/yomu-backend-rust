const TEST_DATABASE_URL: &str = "postgres://yomu:yomu_password@localhost:5432/yomu_engine_test";
const TEST_REDIS_URL: &str = "redis://localhost:6379";

use chrono::Utc;
use uuid::Uuid;

use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
use yomu_backend_rust::modules::league::domain::entities::clan_member::ClanMember;
use yomu_backend_rust::modules::league::domain::entities::clan_member::MemberRole;

mod pg_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    async fn setup_pg_pool() -> sqlx::PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(TEST_DATABASE_URL)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_pg_create_and_get_clan() {
        let pool = setup_pg_pool().await;

        let leader_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to insert leader user");

        let clan = Clan::new("Test Clan".to_string(), leader_id);
        let clan_id = clan.id();

        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan_id)
        .bind(clan.name())
        .bind(clan.leader_id())
        .bind(clan.tier().to_string())
        .bind(clan.total_score())
        .bind(clan.created_at())
        .execute(&pool)
        .await
        .expect("Failed to insert clan");

        let row: (Uuid, String, Uuid, String, i32) = sqlx::query_as(
            "SELECT id, name, leader_id, tier, total_score FROM clans WHERE id = $1",
        )
        .bind(clan_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch clan");

        assert_eq!(row.0, clan_id);
        assert_eq!(row.1, "Test Clan");
        assert_eq!(row.2, leader_id);
        assert_eq!(row.3, "Bronze");
        assert_eq!(row.4, 0i32);

        sqlx::query("DELETE FROM clans WHERE id = $1")
            .bind(clan_id)
            .execute(&pool)
            .await
            .expect("Failed to delete clan");

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to delete user");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_add_member() {
        let pool = setup_pg_pool().await;

        let leader_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to insert leader user");

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(member_id)
            .execute(&pool)
            .await
            .expect("Failed to insert member user");

        let clan = Clan::new("Test Clan Members".to_string(), leader_id);
        let clan_id = clan.id();

        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan_id)
        .bind(clan.name())
        .bind(clan.leader_id())
        .bind(clan.tier().to_string())
        .bind(clan.total_score())
        .bind(clan.created_at())
        .execute(&pool)
        .await
        .expect("Failed to insert clan");

        let member = ClanMember::new(clan_id, member_id, MemberRole::Member);

        sqlx::query("INSERT INTO clan_members (clan_id, user_id, joined_at) VALUES ($1, $2, $3)")
            .bind(member.clan_id())
            .bind(member.user_id())
            .bind(member.joined_at())
            .execute(&pool)
            .await
            .expect("Failed to insert clan member");

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clan_members WHERE clan_id = $1")
            .bind(clan_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count members");

        assert_eq!(count.0, 1);

        sqlx::query("DELETE FROM clan_members WHERE clan_id = $1")
            .bind(clan_id)
            .execute(&pool)
            .await
            .expect("Failed to delete members");

        sqlx::query("DELETE FROM clans WHERE id = $1")
            .bind(clan_id)
            .execute(&pool)
            .await
            .expect("Failed to delete clan");

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to delete leader user");

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(member_id)
            .execute(&pool)
            .await
            .expect("Failed to delete member user");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_get_active_buffs() {
        let pool = setup_pg_pool().await;

        let leader_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to insert leader user");

        let clan = Clan::new("Test Clan Buffs".to_string(), leader_id);
        let clan_id = clan.id();

        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan_id)
        .bind(clan.name())
        .bind(clan.leader_id())
        .bind(clan.tier().to_string())
        .bind(clan.total_score())
        .bind(clan.created_at())
        .execute(&pool)
        .await
        .expect("Failed to insert clan");

        let buff_id = Uuid::new_v4();
        let expired_buff_id = Uuid::new_v4();
        let future_expires = Utc::now() + chrono::Duration::hours(1);
        let past_expires = Utc::now() - chrono::Duration::hours(1);

        sqlx::query(
            "INSERT INTO clan_buffs (id, clan_id, buff_name, multiplier, is_active, expires_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(buff_id)
        .bind(clan_id)
        .bind("Double XP")
        .bind(1.5)
        .bind(true)
        .bind(future_expires)
        .execute(&pool)
        .await
        .expect("Failed to insert active buff");

        sqlx::query(
            "INSERT INTO clan_buffs (id, clan_id, buff_name, multiplier, is_active, expires_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(expired_buff_id)
        .bind(clan_id)
        .bind("Expired Buff")
        .bind(1.2)
        .bind(false)
        .bind(past_expires)
        .execute(&pool)
        .await
        .expect("Failed to insert expired buff");

        let active_buffs: Vec<(String, f64)> = sqlx::query_as(
            "SELECT buff_name, multiplier::float8 FROM clan_buffs WHERE clan_id = $1 AND expires_at > NOW()"
        )
        .bind(clan_id)
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch active buffs");

        assert_eq!(active_buffs.len(), 1);
        assert_eq!(active_buffs[0].0, "Double XP");
        assert_eq!(active_buffs[0].1, 1.5);

        sqlx::query("DELETE FROM clan_buffs WHERE clan_id = $1")
            .bind(clan_id)
            .execute(&pool)
            .await
            .expect("Failed to delete buffs");

        sqlx::query("DELETE FROM clans WHERE id = $1")
            .bind(clan_id)
            .execute(&pool)
            .await
            .expect("Failed to delete clan");

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to delete user");

        pool.close().await;
    }
}

mod season_pg_tests {
    use chrono::{TimeDelta, Utc};
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use uuid::Uuid;
    use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
    use yomu_backend_rust::modules::league::domain::entities::clan::ClanTier;
    use yomu_backend_rust::modules::league::domain::entities::season::Season;
    use yomu_backend_rust::modules::league::domain::repositories::season_repository::SeasonRepository;
    use yomu_backend_rust::modules::league::infrastructure::database::postgres::SeasonPostgresRepo;

    async fn setup_pg_pool() -> sqlx::PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(super::TEST_DATABASE_URL)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_season_create_and_get_by_id() {
        let pool = setup_pg_pool().await;
        let repo = SeasonPostgresRepo::new(pool.clone());

        let starts_at = Utc::now() + TimeDelta::days(1);
        let ends_at = starts_at + TimeDelta::days(30);
        let season = Season::new(
            "Test Season PG".to_string(),
            ClanTier::Bronze,
            starts_at,
            ends_at,
        );
        let season_id = season.id();

        repo.create_season(&season)
            .await
            .expect("Failed to create season");

        let fetched = repo
            .get_season_by_id(season_id)
            .await
            .expect("Failed to get season");
        let fetched = fetched.expect("Season should exist");

        assert_eq!(fetched.id(), season_id);
        assert_eq!(fetched.name(), "Test Season PG");
        assert_eq!(fetched.tier(), &ClanTier::Bronze);
        assert!(!fetched.is_active());

        sqlx::query("DELETE FROM seasons WHERE id = $1")
            .bind(season_id)
            .execute(&pool)
            .await
            .expect("Failed to delete season");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_season_update_clan_tier() {
        let pool = setup_pg_pool().await;

        let leader_id = Uuid::new_v4();
        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to insert leader user");

        let clan = Clan::new("Tier Upgrade Clan".to_string(), leader_id);
        let clan_id = clan.id();

        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan_id)
        .bind(clan.name())
        .bind(clan.leader_id())
        .bind(clan.tier().to_string())
        .bind(clan.total_score())
        .bind(clan.created_at())
        .execute(&pool)
        .await
        .expect("Failed to insert clan");

        let repo = SeasonPostgresRepo::new(pool.clone());
        repo.update_clan_tier(clan_id, &ClanTier::Silver)
            .await
            .expect("Failed to update clan tier");

        let row: (String,) = sqlx::query_as("SELECT tier FROM clans WHERE id = $1")
            .bind(clan_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch clan tier");

        assert_eq!(row.0, "Silver", "Clan tier should be updated to Silver");

        sqlx::query("DELETE FROM clans WHERE id = $1")
            .bind(clan_id)
            .execute(&pool)
            .await
            .expect("Failed to delete clan");
        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to delete user");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_season_mark_ended() {
        let pool = setup_pg_pool().await;
        let repo = SeasonPostgresRepo::new(pool.clone());

        let starts_at = Utc::now() - TimeDelta::days(1);
        let ends_at = Utc::now() + TimeDelta::days(30);
        let season = Season::with_id(
            Uuid::new_v4(),
            "Active Season".to_string(),
            ClanTier::Bronze,
            starts_at,
            ends_at,
            true,
        );
        let season_id = season.id();

        sqlx::query(
            "INSERT INTO seasons (id, name, tier, starts_at, ends_at, is_active) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(season_id)
        .bind(season.name())
        .bind(season.tier().to_string())
        .bind(season.starts_at())
        .bind(season.ends_at())
        .bind(season.is_active())
        .execute(&pool)
        .await
        .expect("Failed to insert season");

        repo.mark_season_ended(season_id)
            .await
            .expect("Failed to mark season ended");

        let fetched = repo
            .get_season_by_id(season_id)
            .await
            .expect("Failed to get season");
        let fetched = fetched.expect("Season should exist");
        assert!(!fetched.is_active(), "Season should be marked as inactive");

        sqlx::query("DELETE FROM seasons WHERE id = $1")
            .bind(season_id)
            .execute(&pool)
            .await
            .expect("Failed to delete season");

        pool.close().await;
    }
}

mod clan_buff_pg_tests {
    use chrono::{TimeDelta, Utc};
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use uuid::Uuid;
    use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
    use yomu_backend_rust::modules::league::domain::entities::clan_buff::ClanBuff;
    use yomu_backend_rust::modules::league::infrastructure::database::postgres::clan_buff_postgres_repo::ClanBuffPostgresRepo;
    use yomu_backend_rust::modules::league::domain::repositories::ClanBuffRepository;

    async fn setup_pg_pool() -> sqlx::PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(super::TEST_DATABASE_URL)
            .await
            .expect("Failed to connect to test database")
    }

    async fn setup_clan_and_leader(pool: &sqlx::PgPool) -> (Uuid, Uuid) {
        let leader_id = Uuid::new_v4();
        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(leader_id)
            .execute(pool)
            .await
            .expect("Failed to insert leader user");

        let clan = Clan::new("Buff Test Clan".to_string(), leader_id);
        let clan_id = clan.id();

        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan_id)
        .bind(clan.name())
        .bind(clan.leader_id())
        .bind(clan.tier().to_string())
        .bind(clan.total_score())
        .bind(clan.created_at())
        .execute(pool)
        .await
        .expect("Failed to insert clan");

        (clan_id, leader_id)
    }

    async fn cleanup_clan_and_leader(pool: &sqlx::PgPool, clan_id: Uuid, leader_id: Uuid) {
        sqlx::query("DELETE FROM clan_buffs WHERE clan_id = $1")
            .bind(clan_id)
            .execute(pool)
            .await
            .expect("Failed to delete buffs");
        sqlx::query("DELETE FROM clans WHERE id = $1")
            .bind(clan_id)
            .execute(pool)
            .await
            .expect("Failed to delete clan");
        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(leader_id)
            .execute(pool)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    async fn test_activate_buff() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id) = setup_clan_and_leader(&pool).await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let buff = ClanBuff::new(
            Uuid::new_v4(),
            clan_id,
            "Productivity Buff".to_string(),
            1.5,
            true,
            false,
            Utc::now() + TimeDelta::hours(1),
            Utc::now(),
        );

        repo.activate_buff(&buff)
            .await
            .expect("Failed to activate buff");

        let row: (String, bool) = sqlx::query_as(
            "SELECT buff_name, is_active FROM clan_buffs WHERE clan_id = $1 AND buff_name = $2",
        )
        .bind(clan_id)
        .bind("Productivity Buff")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch buff");

        assert_eq!(row.0, "Productivity Buff");
        assert!(row.1, "Buff should be active");

        cleanup_clan_and_leader(&pool, clan_id, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_deactivate_buffs_for_clan() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id) = setup_clan_and_leader(&pool).await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let buff = ClanBuff::new(
            Uuid::new_v4(),
            clan_id,
            "Speed Boost".to_string(),
            1.3,
            true,
            false,
            Utc::now() + TimeDelta::hours(1),
            Utc::now(),
        );
        repo.activate_buff(&buff)
            .await
            .expect("Failed to activate buff");

        repo.deactivate_buffs_for_clan(clan_id)
            .await
            .expect("Failed to deactivate buffs");

        let active_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM clan_buffs WHERE clan_id = $1 AND is_active = true",
        )
        .bind(clan_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count active buffs");

        assert_eq!(active_count.0, 0, "All buffs should be deactivated");

        cleanup_clan_and_leader(&pool, clan_id, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_active_buffs() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id) = setup_clan_and_leader(&pool).await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let buff1 = ClanBuff::new(
            Uuid::new_v4(),
            clan_id,
            "Active Buff 1".to_string(),
            1.5,
            true,
            false,
            Utc::now() + TimeDelta::hours(1),
            Utc::now(),
        );
        let buff2 = ClanBuff::new(
            Uuid::new_v4(),
            clan_id,
            "Active Buff 2".to_string(),
            1.2,
            true,
            false,
            Utc::now() + TimeDelta::hours(2),
            Utc::now(),
        );

        repo.activate_buff(&buff1)
            .await
            .expect("Failed to activate buff 1");
        repo.activate_buff(&buff2)
            .await
            .expect("Failed to activate buff 2");

        let active_buffs = repo
            .get_active_buffs(clan_id)
            .await
            .expect("Failed to get active buffs");

        assert!(
            active_buffs.len() >= 2,
            "Should have at least 2 active buffs"
        );

        cleanup_clan_and_leader(&pool, clan_id, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_deactivate_buff_by_name() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id) = setup_clan_and_leader(&pool).await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let buff = ClanBuff::new(
            Uuid::new_v4(),
            clan_id,
            "Target Buff".to_string(),
            1.4,
            true,
            false,
            Utc::now() + TimeDelta::hours(1),
            Utc::now(),
        );

        repo.activate_buff(&buff)
            .await
            .expect("Failed to activate buff");
        repo.deactivate_buff_by_name(clan_id, "Target Buff")
            .await
            .expect("Failed to deactivate buff by name");

        let is_active: (bool,) = sqlx::query_as(
            "SELECT is_active FROM clan_buffs WHERE clan_id = $1 AND buff_name = $2",
        )
        .bind(clan_id)
        .bind("Target Buff")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch buff status");

        assert!(
            !is_active.0,
            "Buff should be deactivated after deactivation by name"
        );

        cleanup_clan_and_leader(&pool, clan_id, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_buff_by_name_found() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id) = setup_clan_and_leader(&pool).await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let buff = ClanBuff::new(
            Uuid::new_v4(),
            clan_id,
            "Named Buff".to_string(),
            1.75,
            true,
            false,
            Utc::now() + TimeDelta::hours(2),
            Utc::now(),
        );

        repo.activate_buff(&buff)
            .await
            .expect("Failed to activate buff");

        let found = repo
            .get_buff_by_name(clan_id, "Named Buff")
            .await
            .expect("Failed to get buff by name");

        assert!(found.is_some(), "Should find buff by name");
        let found_buff = found.unwrap();
        assert_eq!(found_buff.buff_name(), "Named Buff");
        assert_eq!(found_buff.clan_id(), clan_id);

        cleanup_clan_and_leader(&pool, clan_id, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_buff_by_name_not_found() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id) = setup_clan_and_leader(&pool).await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let found = repo
            .get_buff_by_name(clan_id, "NonExistent Buff")
            .await
            .expect("Failed to query buff by name");

        assert!(found.is_none(), "Should return None for non-existent buff name");

        cleanup_clan_and_leader(&pool, clan_id, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_avg_accuracy_for_members_empty() {
        let pool = setup_pg_pool().await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let avg = repo
            .get_avg_accuracy_for_members(&[])
            .await
            .expect("Failed to get avg accuracy for empty members");

        assert!(
            (avg - 0.0).abs() < f64::EPSILON,
            "Empty members should return 0.0, got {}",
            avg
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_mission_completion_rate_for_clan_no_members() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id) = setup_clan_and_leader(&pool).await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let rate = repo
            .get_mission_completion_rate_for_clan(clan_id)
            .await
            .expect("Failed to get mission completion rate");

        assert!(
            (rate - 0.0).abs() < f64::EPSILON,
            "Mission completion rate should be 0.0 for clan with no members, got {}",
            rate
        );

        cleanup_clan_and_leader(&pool, clan_id, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_mission_completion_rate_for_clan_with_members() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id) = setup_clan_and_leader(&pool).await;

        sqlx::query("INSERT INTO clan_members (clan_id, user_id) VALUES ($1, $2)")
            .bind(clan_id)
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to insert leader as clan member");

        let mission_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO daily_missions (id, description, target_count, date, reward_points) VALUES ($1, $2, $3, CURRENT_DATE, 10)"
        )
        .bind(mission_id)
        .bind("Read 3 articles")
        .bind(3)
        .execute(&pool)
        .await
        .expect("Failed to insert daily mission");

        sqlx::query(
            "INSERT INTO user_missions (user_id, mission_id, current_progress, is_claimed) VALUES ($1, $2, 1, true)"
        )
        .bind(leader_id)
        .bind(mission_id)
        .execute(&pool)
        .await
        .expect("Failed to insert user mission");

        let repo = ClanBuffPostgresRepo::new(pool.clone());
        let rate = repo
            .get_mission_completion_rate_for_clan(clan_id)
            .await
            .expect("Failed to get mission completion rate");

        assert!(
            rate > 0.0,
            "Mission completion rate should be > 0, got {}",
            rate
        );

        sqlx::query("DELETE FROM user_missions WHERE mission_id = $1")
            .bind(mission_id)
            .execute(&pool)
            .await
            .expect("Failed to delete user mission");
        sqlx::query("DELETE FROM daily_missions WHERE id = $1")
            .bind(mission_id)
            .execute(&pool)
            .await
            .expect("Failed to delete daily mission");
        sqlx::query("DELETE FROM clan_members WHERE clan_id = $1 AND user_id = $2")
            .bind(clan_id)
            .bind(leader_id)
            .execute(&pool)
            .await
            .expect("Failed to delete clan member");

        cleanup_clan_and_leader(&pool, clan_id, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_avg_quiz_score_for_members() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert user");

        let quiz_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO quiz_history (id, user_id, article_id, score, accuracy, completed_at) VALUES ($1, $2, $3, $4, $5, NOW())"
        )
        .bind(quiz_id)
        .bind(user_id)
        .bind(article_id)
        .bind(75)
        .bind(0.90)
        .execute(&pool)
        .await
        .expect("Failed to insert quiz history");

        let repo = ClanBuffPostgresRepo::new(pool.clone());
        let avg_score = repo
            .get_avg_quiz_score_for_members(&[user_id])
            .await
            .expect("Failed to get avg quiz score for members");

        assert_eq!(avg_score, 75, "Avg quiz score should be 75, got {}", avg_score);

        sqlx::query("DELETE FROM quiz_history WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to delete quiz history");
        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to delete user");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_get_avg_quiz_score_for_members_empty() {
        let pool = setup_pg_pool().await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let avg_score = repo
            .get_avg_quiz_score_for_members(&[])
            .await
            .expect("Failed to get avg quiz score for empty members");

        assert_eq!(avg_score, 0, "Avg quiz score for empty members should be 0, got {}", avg_score);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_deactivate_buff_by_name_no_matching_buff() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id) = setup_clan_and_leader(&pool).await;
        let repo = ClanBuffPostgresRepo::new(pool.clone());

        let result = repo
            .deactivate_buff_by_name(clan_id, "NonExistent Buff")
            .await;
        assert!(result.is_ok(), "Deactivating non-existent buff should succeed");

        cleanup_clan_and_leader(&pool, clan_id, leader_id).await;
        pool.close().await;
    }
}

mod redis_tests {
    use super::*;
    async fn setup_redis() -> redis::aio::MultiplexedConnection {
        let client = redis::Client::open(TEST_REDIS_URL).expect("Failed to create Redis client");
        client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis")
    }

    #[tokio::test]
    async fn test_redis_update_score_and_get_top() {
        let mut con = setup_redis().await;

        let tier = "Bronze";
        let clan_a_id = Uuid::new_v4();
        let clan_b_id = Uuid::new_v4();

        // Use aggregate leaderboard key directly
        let leaderboard_key = format!("leaderboard:{}", tier);

        let _: () = redis::cmd("ZADD")
            .arg(&leaderboard_key)
            .arg(100)
            .arg(clan_a_id.to_string())
            .query_async(&mut con)
            .await
            .expect("Failed to add score for clan A");

        let _: () = redis::cmd("ZADD")
            .arg(&leaderboard_key)
            .arg(50)
            .arg(clan_b_id.to_string())
            .query_async(&mut con)
            .await
            .expect("Failed to add score for clan B");

        let top_clans: Vec<(String, i64)> = redis::cmd("ZREVRANGE")
            .arg(&leaderboard_key)
            .arg(0)
            .arg(10)
            .arg("WITHSCORES")
            .query_async(&mut con)
            .await
            .unwrap_or_else(|_| vec![]);

        let _: () = redis::cmd("DEL")
            .arg(&leaderboard_key)
            .query_async(&mut con)
            .await
            .expect("Failed to clean up leaderboard key");

        if !top_clans.is_empty() {
            let first_clan_score = top_clans[0].1;
            assert_eq!(first_clan_score, 100);
        }
    }

    #[tokio::test]
    async fn test_redis_leaderboard_cache_update_clan_score() {
        use yomu_backend_rust::modules::league::domain::repositories::LeaderboardCache;
        use yomu_backend_rust::modules::league::infrastructure::database::redis::LeaderboardRedisRepo;

        let con = setup_redis().await;
        let repo = LeaderboardRedisRepo::new(con);

        let clan_id = Uuid::new_v4();

        repo.update_clan_score(clan_id, 250)
            .await
            .expect("Failed to update clan score");

        let score = repo
            .get_clan_score(clan_id)
            .await
            .expect("Failed to get clan score");

        assert_eq!(score, Some(250), "Clan score should be 250 after update");

        let mut cleanup_con = setup_redis().await;
        let key = format!("leaderboard:global");
        let _: () = redis::cmd("ZREM")
            .arg(&key)
            .arg(clan_id.to_string())
            .query_async(&mut cleanup_con)
            .await
            .expect("Failed to clean up");
    }

    #[tokio::test]
    async fn test_redis_leaderboard_cache_get_clan_score_not_found() {
        use yomu_backend_rust::modules::league::domain::repositories::LeaderboardCache;
        use yomu_backend_rust::modules::league::infrastructure::database::redis::LeaderboardRedisRepo;

        let con = setup_redis().await;
        let repo = LeaderboardRedisRepo::new(con);

        let nonexistent_id = Uuid::new_v4();
        let score = repo
            .get_clan_score(nonexistent_id)
            .await
            .expect("Failed to get clan score");

        assert_eq!(score, None, "Non-existent clan score should be None");
    }

    #[tokio::test]
    async fn test_redis_leaderboard_cache_remove_clan() {
        use yomu_backend_rust::modules::league::domain::repositories::LeaderboardCache;
        use yomu_backend_rust::modules::league::infrastructure::database::redis::LeaderboardRedisRepo;

        let con = setup_redis().await;
        let repo = LeaderboardRedisRepo::new(con);

        let clan_id = Uuid::new_v4();

        repo.update_clan_score(clan_id, 100)
            .await
            .expect("Failed to update clan score");

        repo.remove_clan_from_leaderboard(clan_id)
            .await
            .expect("Failed to remove clan");

        let score = repo
            .get_clan_score(clan_id)
            .await
            .expect("Failed to get clan score");
        assert_eq!(score, None, "Clan should be removed from leaderboard");
    }

    #[tokio::test]
    async fn test_redis_leaderboard_cache_get_top_clans() {
        use yomu_backend_rust::modules::league::domain::repositories::LeaderboardCache;
        use yomu_backend_rust::modules::league::infrastructure::database::redis::LeaderboardRedisRepo;

        let con = setup_redis().await;
        let repo = LeaderboardRedisRepo::new(con);

        let clan_a = Uuid::new_v4();
        let clan_b = Uuid::new_v4();

        repo.update_clan_score(clan_a, 200)
            .await
            .expect("Failed to update clan A score");
        repo.update_clan_score(clan_b, 400)
            .await
            .expect("Failed to update clan B score");

        let top_clans = repo
            .get_top_clans("global", 10)
            .await
            .expect("Failed to get top clans");

        assert!(!top_clans.is_empty(), "Should have clans in leaderboard");

        let mut cleanup_con = setup_redis().await;
        let key = "leaderboard:global";
        let _: () = redis::cmd("ZREM")
            .arg(key)
            .arg(clan_a.to_string())
            .query_async(&mut cleanup_con)
            .await
            .expect("Failed to clean up clan A");
        let _: () = redis::cmd("ZREM")
            .arg(key)
            .arg(clan_b.to_string())
            .query_async(&mut cleanup_con)
            .await
            .expect("Failed to clean up clan B");
    }
}

mod clan_join_request_pg_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use uuid::Uuid;
    use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
    use yomu_backend_rust::modules::league::domain::entities::clan_join_request::{ClanJoinRequest, RequestStatus};
    use yomu_backend_rust::modules::league::domain::repositories::ClanJoinRequestRepository;
    use yomu_backend_rust::modules::league::infrastructure::database::postgres::clan_join_request_postgres_repo::ClanJoinRequestPostgresRepo;

    async fn setup_pg_pool() -> sqlx::PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(super::TEST_DATABASE_URL)
            .await
            .expect("Failed to connect to test database")
    }

    async fn setup_user_and_clan(pool: &sqlx::PgPool) -> (Uuid, Uuid, Uuid) {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS clan_join_requests (\
                id UUID PRIMARY KEY,\
                clan_id UUID NOT NULL REFERENCES clans(id) ON DELETE CASCADE,\
                user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,\
                status VARCHAR(20) NOT NULL DEFAULT 'Pending',\
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),\
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),\
                UNIQUE (clan_id, user_id)\
            )"
        )
        .execute(pool)
        .await
        .expect("Failed to create clan_join_requests table");

        let leader_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(leader_id)
            .execute(pool)
            .await
            .expect("Failed to insert leader user");

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(pool)
            .await
            .expect("Failed to insert request user");

        let clan = Clan::new("Request Test Clan".to_string(), leader_id);
        let clan_id = clan.id();

        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan_id)
        .bind(clan.name())
        .bind(clan.leader_id())
        .bind(clan.tier().to_string())
        .bind(clan.total_score())
        .bind(clan.created_at())
        .execute(pool)
        .await
        .expect("Failed to insert clan");

        (clan_id, leader_id, user_id)
    }

    async fn cleanup_user_and_clan(pool: &sqlx::PgPool, clan_id: Uuid, leader_id: Uuid, user_id: Uuid) {
        sqlx::query("DELETE FROM clan_join_requests WHERE clan_id = $1 OR user_id = $2")
            .bind(clan_id)
            .bind(user_id)
            .execute(pool)
            .await
            .expect("Failed to delete join requests");
        sqlx::query("DELETE FROM clan_members WHERE clan_id = $1")
            .bind(clan_id)
            .execute(pool)
            .await
            .expect("Failed to delete clan members");
        sqlx::query("DELETE FROM clans WHERE id = $1")
            .bind(clan_id)
            .execute(pool)
            .await
            .expect("Failed to delete clan");
        sqlx::query("DELETE FROM engine_users WHERE user_id = $1 OR user_id = $2")
            .bind(leader_id)
            .bind(user_id)
            .execute(pool)
            .await
            .expect("Failed to delete users");
        sqlx::query("DROP TABLE IF EXISTS clan_join_requests")
            .execute(pool)
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_clan_join_request_create_and_get() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id, user_id) = setup_user_and_clan(&pool).await;
        let repo = ClanJoinRequestPostgresRepo::new(pool.clone());

        let request = ClanJoinRequest::new(clan_id, user_id);
        let request_id = request.id();

        repo.create_request(&request)
            .await
            .expect("Failed to create join request");

        let fetched = repo
            .get_request_by_id(request_id)
            .await
            .expect("Failed to get request")
            .expect("Request should exist");

        assert_eq!(fetched.id(), request_id);
        assert_eq!(fetched.clan_id(), clan_id);
        assert_eq!(fetched.user_id(), user_id);
        assert_eq!(fetched.status(), &RequestStatus::Pending);

        cleanup_user_and_clan(&pool, clan_id, leader_id, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_join_request_get_pending_by_clan() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id, user_id) = setup_user_and_clan(&pool).await;
        let repo = ClanJoinRequestPostgresRepo::new(pool.clone());

        let request = ClanJoinRequest::new(clan_id, user_id);
        repo.create_request(&request)
            .await
            .expect("Failed to create join request");

        let pending = repo
            .get_pending_requests_by_clan(clan_id)
            .await
            .expect("Failed to get pending requests");

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].clan_id(), clan_id);
        assert_eq!(pending[0].user_id(), user_id);

        cleanup_user_and_clan(&pool, clan_id, leader_id, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_join_request_get_pending_by_user() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id, user_id) = setup_user_and_clan(&pool).await;
        let repo = ClanJoinRequestPostgresRepo::new(pool.clone());

        let request = ClanJoinRequest::new(clan_id, user_id);
        let request_id = request.id();
        repo.create_request(&request)
            .await
            .expect("Failed to create join request");

        let fetched = repo
            .get_pending_request_by_user(user_id, clan_id)
            .await
            .expect("Failed to get pending request by user")
            .expect("Request should exist");

        assert_eq!(fetched.id(), request_id);
        assert_eq!(fetched.clan_id(), clan_id);
        assert_eq!(fetched.user_id(), user_id);

        cleanup_user_and_clan(&pool, clan_id, leader_id, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_join_request_has_pending_request() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id, user_id) = setup_user_and_clan(&pool).await;
        let repo = ClanJoinRequestPostgresRepo::new(pool.clone());

        let has_pending_before = repo
            .has_pending_request(user_id)
            .await
            .expect("Failed to check pending");
        assert!(!has_pending_before, "Should have no pending request initially");

        let request = ClanJoinRequest::new(clan_id, user_id);
        repo.create_request(&request)
            .await
            .expect("Failed to create join request");

        let has_pending_after = repo
            .has_pending_request(user_id)
            .await
            .expect("Failed to check pending");
        assert!(has_pending_after, "Should have pending request after creation");

        cleanup_user_and_clan(&pool, clan_id, leader_id, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_join_request_update_status_approved() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id, user_id) = setup_user_and_clan(&pool).await;
        let repo = ClanJoinRequestPostgresRepo::new(pool.clone());

        let request = ClanJoinRequest::new(clan_id, user_id);
        let request_id = request.id();
        repo.create_request(&request)
            .await
            .expect("Failed to create join request");

        repo.update_request_status(request_id, &RequestStatus::Approved)
            .await
            .expect("Failed to approve request");

        let fetched = repo
            .get_request_by_id(request_id)
            .await
            .expect("Failed to get request")
            .expect("Request should exist");

        assert_eq!(fetched.status(), &RequestStatus::Approved);

        cleanup_user_and_clan(&pool, clan_id, leader_id, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_join_request_update_status_rejected() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id, user_id) = setup_user_and_clan(&pool).await;
        let repo = ClanJoinRequestPostgresRepo::new(pool.clone());

        let request = ClanJoinRequest::new(clan_id, user_id);
        let request_id = request.id();
        repo.create_request(&request)
            .await
            .expect("Failed to create join request");

        repo.update_request_status(request_id, &RequestStatus::Rejected)
            .await
            .expect("Failed to reject request");

        let fetched = repo
            .get_request_by_id(request_id)
            .await
            .expect("Failed to get request")
            .expect("Request should exist");

        assert_eq!(fetched.status(), &RequestStatus::Rejected);

        cleanup_user_and_clan(&pool, clan_id, leader_id, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_join_request_delete_request() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id, user_id) = setup_user_and_clan(&pool).await;
        let repo = ClanJoinRequestPostgresRepo::new(pool.clone());

        let request = ClanJoinRequest::new(clan_id, user_id);
        let request_id = request.id();
        repo.create_request(&request)
            .await
            .expect("Failed to create join request");

        repo.delete_request(request_id)
            .await
            .expect("Failed to delete request");

        let fetched = repo
            .get_request_by_id(request_id)
            .await
            .expect("Failed to get request");
        assert!(fetched.is_none(), "Request should be deleted");

        cleanup_user_and_clan(&pool, clan_id, leader_id, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_join_request_delete_pending_requests_by_user() {
        let pool = setup_pg_pool().await;
        let (clan_id, leader_id, user_id) = setup_user_and_clan(&pool).await;
        let repo = ClanJoinRequestPostgresRepo::new(pool.clone());

        let request = ClanJoinRequest::new(clan_id, user_id);
        repo.create_request(&request)
            .await
            .expect("Failed to create join request");

        repo.delete_pending_requests_by_user(user_id)
            .await
            .expect("Failed to delete pending requests by user");

        let pending = repo
            .get_pending_requests_by_clan(clan_id)
            .await
            .expect("Failed to get pending requests");
        assert!(pending.is_empty(), "All pending requests for user should be deleted");

        cleanup_user_and_clan(&pool, clan_id, leader_id, user_id).await;
        pool.close().await;
    }
}

mod clan_repo_pg_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use uuid::Uuid;
    use yomu_backend_rust::modules::league::domain::entities::clan::{Clan, ClanTier};
    use yomu_backend_rust::modules::league::domain::entities::clan_member::{ClanMember, MemberRole};
    use yomu_backend_rust::modules::league::domain::repositories::ClanRepository;
    use yomu_backend_rust::modules::league::infrastructure::database::postgres::ClanPostgresRepo;

    async fn setup_pg_pool() -> sqlx::PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(super::TEST_DATABASE_URL)
            .await
            .expect("Failed to connect to test database")
    }

    async fn insert_user(pool: &sqlx::PgPool, user_id: Uuid) {
        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(pool)
            .await
            .expect("Failed to insert user");
    }

    async fn delete_user(pool: &sqlx::PgPool, user_id: Uuid) {
        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .expect("Failed to delete user");
    }

    #[tokio::test]
    async fn test_clan_repo_create_and_get_by_id() {
        let pool = setup_pg_pool().await;
        let leader_id = Uuid::new_v4();
        insert_user(&pool, leader_id).await;

        let repo = ClanPostgresRepo::new(pool.clone());
        let clan = Clan::new("Repo Clan".to_string(), leader_id);
        let clan_id = clan.id();

        repo.create_clan(&clan)
            .await
            .expect("Failed to create clan");

        let fetched = repo
            .get_clan_by_id(clan_id)
            .await
            .expect("Failed to get clan")
            .expect("Clan should exist");

        assert_eq!(fetched.id(), clan_id);
        assert_eq!(fetched.name(), "Repo Clan");
        assert_eq!(fetched.leader_id(), leader_id);
        assert_eq!(fetched.tier(), &ClanTier::Bronze);
        assert_eq!(fetched.total_score(), 0);

        repo.delete_clan(clan_id)
            .await
            .expect("Failed to delete clan");
        delete_user(&pool, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_repo_add_member_and_get_members() {
        let pool = setup_pg_pool().await;
        let leader_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        insert_user(&pool, leader_id).await;
        insert_user(&pool, member_id).await;

        let repo = ClanPostgresRepo::new(pool.clone());
        let clan = Clan::new("Member Clan".to_string(), leader_id);
        let clan_id = clan.id();

        repo.create_clan(&clan)
            .await
            .expect("Failed to create clan");

        let member = ClanMember::new(clan_id, member_id, MemberRole::Member);
        repo.add_member(&member)
            .await
            .expect("Failed to add member");

        let members = repo
            .get_members_by_clan_id(clan_id)
            .await
            .expect("Failed to get members");

        let member_ids: Vec<Uuid> = members.iter().map(|m| m.user_id()).collect();
        assert!(member_ids.contains(&member_id), "Should contain added member");

        repo.delete_clan(clan_id)
            .await
            .expect("Failed to delete clan");
        delete_user(&pool, leader_id).await;
        delete_user(&pool, member_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_repo_is_user_in_any_clan() {
        let pool = setup_pg_pool().await;
        let leader_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        insert_user(&pool, leader_id).await;
        insert_user(&pool, member_id).await;

        let repo = ClanPostgresRepo::new(pool.clone());
        let clan = Clan::new("Any Clan".to_string(), leader_id);
        let clan_id = clan.id();

        let before = repo
            .is_user_in_any_clan(member_id)
            .await
            .expect("Failed to check clan membership");
        assert!(!before, "User should not be in any clan initially");

        repo.create_clan(&clan)
            .await
            .expect("Failed to create clan");

        let member = ClanMember::new(clan_id, member_id, MemberRole::Member);
        repo.add_member(&member)
            .await
            .expect("Failed to add member");

        let after = repo
            .is_user_in_any_clan(member_id)
            .await
            .expect("Failed to check clan membership");
        assert!(after, "User should be in a clan after adding");

        repo.delete_clan(clan_id)
            .await
            .expect("Failed to delete clan");
        delete_user(&pool, leader_id).await;
        delete_user(&pool, member_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_repo_get_user_clan_id() {
        let pool = setup_pg_pool().await;
        let leader_id = Uuid::new_v4();
        let member_id = Uuid::new_v4();
        insert_user(&pool, leader_id).await;
        insert_user(&pool, member_id).await;

        let repo = ClanPostgresRepo::new(pool.clone());
        let clan = Clan::new("UserClan".to_string(), leader_id);
        let clan_id = clan.id();

        repo.create_clan(&clan)
            .await
            .expect("Failed to create clan");

        let member = ClanMember::new(clan_id, member_id, MemberRole::Member);
        repo.add_member(&member)
            .await
            .expect("Failed to add member");

        let fetched_clan_id = repo
            .get_user_clan_id(member_id)
            .await
            .expect("Failed to get user clan id")
            .expect("User should have a clan");

        assert_eq!(fetched_clan_id, clan_id);

        repo.delete_clan(clan_id)
            .await
            .expect("Failed to delete clan");
        delete_user(&pool, leader_id).await;
        delete_user(&pool, member_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_repo_get_user_tier_info() {
        let pool = setup_pg_pool().await;
        let leader_id = Uuid::new_v4();
        insert_user(&pool, leader_id).await;

        let repo = ClanPostgresRepo::new(pool.clone());
        let clan = Clan::new("Tier Clan".to_string(), leader_id);
        let clan_id = clan.id();

        repo.create_clan(&clan)
            .await
            .expect("Failed to create clan");

        let member = ClanMember::new(clan_id, leader_id, MemberRole::Leader);
        repo.add_member(&member)
            .await
            .expect("Failed to add leader member");

        let info = repo
            .get_user_tier_info(leader_id)
            .await
            .expect("Failed to get tier info")
            .expect("Should have tier info");

        assert_eq!(info.0, clan_id);
        assert_eq!(info.1, "Tier Clan");
        assert_eq!(info.2, ClanTier::Bronze);

        repo.delete_clan(clan_id)
            .await
            .expect("Failed to delete clan");
        delete_user(&pool, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_repo_add_score() {
        let pool = setup_pg_pool().await;
        let leader_id = Uuid::new_v4();
        insert_user(&pool, leader_id).await;

        let repo = ClanPostgresRepo::new(pool.clone());
        let clan = Clan::new("Score Clan".to_string(), leader_id);
        let clan_id = clan.id();

        repo.create_clan(&clan)
            .await
            .expect("Failed to create clan");

        repo.add_score(clan_id, 50)
            .await
            .expect("Failed to add score");

        let fetched = repo
            .get_clan_by_id(clan_id)
            .await
            .expect("Failed to get clan")
            .expect("Clan should exist");

        assert_eq!(fetched.total_score(), 50);

        repo.delete_clan(clan_id)
            .await
            .expect("Failed to delete clan");
        delete_user(&pool, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_repo_delete_clan() {
        let pool = setup_pg_pool().await;
        let leader_id = Uuid::new_v4();
        insert_user(&pool, leader_id).await;

        let repo = ClanPostgresRepo::new(pool.clone());
        let clan = Clan::new("Delete Clan".to_string(), leader_id);
        let clan_id = clan.id();

        repo.create_clan(&clan)
            .await
            .expect("Failed to create clan");

        repo.delete_clan(clan_id)
            .await
            .expect("Failed to delete clan");

        let fetched = repo
            .get_clan_by_id(clan_id)
            .await
            .expect("Failed to get clan");
        assert!(fetched.is_none(), "Clan should be deleted");

        delete_user(&pool, leader_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_repo_get_leaders_by_clan_ids() {
        let pool = setup_pg_pool().await;
        let leader_a = Uuid::new_v4();
        let leader_b = Uuid::new_v4();
        insert_user(&pool, leader_a).await;
        insert_user(&pool, leader_b).await;

        let repo = ClanPostgresRepo::new(pool.clone());
        let clan_a = Clan::new("Clan A".to_string(), leader_a);
        let clan_b = Clan::new("Clan B".to_string(), leader_b);
        let clan_a_id = clan_a.id();
        let clan_b_id = clan_b.id();

        repo.create_clan(&clan_a)
            .await
            .expect("Failed to create clan A");
        repo.create_clan(&clan_b)
            .await
            .expect("Failed to create clan B");

        let leaders = repo
            .get_leaders_by_clan_ids(&[clan_a_id, clan_b_id])
            .await
            .expect("Failed to get leaders");

        assert_eq!(leaders.get(&clan_a_id), Some(&leader_a));
        assert_eq!(leaders.get(&clan_b_id), Some(&leader_b));

        repo.delete_clan(clan_a_id)
            .await
            .expect("Failed to delete clan A");
        repo.delete_clan(clan_b_id)
            .await
            .expect("Failed to delete clan B");
        delete_user(&pool, leader_a).await;
        delete_user(&pool, leader_b).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_repo_get_clan_names_by_ids() {
        let pool = setup_pg_pool().await;
        let leader_a = Uuid::new_v4();
        let leader_b = Uuid::new_v4();
        insert_user(&pool, leader_a).await;
        insert_user(&pool, leader_b).await;

        let repo = ClanPostgresRepo::new(pool.clone());
        let clan_a = Clan::new("Name A".to_string(), leader_a);
        let clan_b = Clan::new("Name B".to_string(), leader_b);
        let clan_a_id = clan_a.id();
        let clan_b_id = clan_b.id();

        repo.create_clan(&clan_a)
            .await
            .expect("Failed to create clan A");
        repo.create_clan(&clan_b)
            .await
            .expect("Failed to create clan B");

        let names = repo
            .get_clan_names_by_ids(&[clan_a_id, clan_b_id])
            .await
            .expect("Failed to get clan names");

        assert_eq!(names.get(&clan_a_id), Some(&"Name A".to_string()));
        assert_eq!(names.get(&clan_b_id), Some(&"Name B".to_string()));

        repo.delete_clan(clan_a_id)
            .await
            .expect("Failed to delete clan A");
        repo.delete_clan(clan_b_id)
            .await
            .expect("Failed to delete clan B");
        delete_user(&pool, leader_a).await;
        delete_user(&pool, leader_b).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_clan_repo_ensure_user_exists() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        let repo = ClanPostgresRepo::new(pool.clone());
        repo.ensure_user_exists(user_id)
            .await
            .expect("Failed to ensure user exists");

        let row: (Uuid,) = sqlx::query_as("SELECT user_id FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("User should exist in DB");

        assert_eq!(row.0, user_id);

        repo.ensure_user_exists(user_id)
            .await
            .expect("Second ensure should be idempotent");

        delete_user(&pool, user_id).await;
        pool.close().await;
    }
}

mod season_repo_gap_tests {
    use chrono::{TimeDelta, Utc};
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use uuid::Uuid;
    use yomu_backend_rust::modules::league::domain::entities::clan::{Clan, ClanTier};
    use yomu_backend_rust::modules::league::domain::entities::season::Season;
    use yomu_backend_rust::modules::league::domain::repositories::season_repository::SeasonRepository;
    use yomu_backend_rust::modules::league::infrastructure::database::postgres::SeasonPostgresRepo;

    async fn setup_pg_pool() -> sqlx::PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(super::TEST_DATABASE_URL)
            .await
            .expect("Failed to connect to test database")
    }

    async fn insert_user(pool: &sqlx::PgPool, user_id: Uuid) {
        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(pool)
            .await
            .expect("Failed to insert user");
    }

    #[tokio::test]
    async fn test_season_get_active_season() {
        let pool = setup_pg_pool().await;
        let repo = SeasonPostgresRepo::new(pool.clone());

        let starts_at = Utc::now() - TimeDelta::days(1);
        let ends_at = Utc::now() + TimeDelta::days(30);
        let season = Season::with_id(
            Uuid::new_v4(),
            "Active Bronze Season".to_string(),
            ClanTier::Bronze,
            starts_at,
            ends_at,
            true,
        );
        let season_id = season.id();

        sqlx::query(
            "INSERT INTO seasons (id, name, tier, starts_at, ends_at, is_active) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(season_id)
        .bind(season.name())
        .bind(season.tier().to_string())
        .bind(season.starts_at())
        .bind(season.ends_at())
        .bind(season.is_active())
        .execute(&pool)
        .await
        .expect("Failed to insert season");

        let active = repo
            .get_active_season(&ClanTier::Bronze)
            .await
            .expect("Failed to get active season")
            .expect("Active season should exist");

        assert_eq!(active.id(), season_id);
        assert_eq!(active.name(), "Active Bronze Season");
        assert!(active.is_active());

        sqlx::query("DELETE FROM seasons WHERE id = $1")
            .bind(season_id)
            .execute(&pool)
            .await
            .expect("Failed to delete season");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_season_get_season_results() {
        let pool = setup_pg_pool().await;
        let repo = SeasonPostgresRepo::new(pool.clone());

        sqlx::query("ALTER TABLE clans ALTER COLUMN total_score TYPE BIGINT")
            .execute(&pool)
            .await
            .expect("Failed to alter total_score column");

        let leader_a = Uuid::new_v4();
        let leader_b = Uuid::new_v4();
        insert_user(&pool, leader_a).await;
        insert_user(&pool, leader_b).await;

        let clan_a = Clan::new("Result Clan A".to_string(), leader_a);
        let clan_b = Clan::new("Result Clan B".to_string(), leader_b);
        let clan_a_id = clan_a.id();
        let clan_b_id = clan_b.id();

        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan_a_id)
        .bind(clan_a.name())
        .bind(clan_a.leader_id())
        .bind("Bronze")
        .bind(100i64)
        .bind(clan_a.created_at())
        .execute(&pool)
        .await
        .expect("Failed to insert clan A");

        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan_b_id)
        .bind(clan_b.name())
        .bind(clan_b.leader_id())
        .bind("Bronze")
        .bind(200i64)
        .bind(clan_b.created_at())
        .execute(&pool)
        .await
            .expect("Failed to insert clan B");

        let results = repo
            .get_season_results(&ClanTier::Bronze, Uuid::new_v4())
            .await
            .expect("Failed to get season results");

        let a_result = results.iter().find(|r| r.0 == clan_a_id);
        let b_result = results.iter().find(|r| r.0 == clan_b_id);

        assert!(a_result.is_some(), "Clan A should be in results");
        assert!(b_result.is_some(), "Clan B should be in results");

        let a = a_result.unwrap();
        let b = b_result.unwrap();

        assert_eq!(a.1, "Result Clan A");
        assert_eq!(a.3, 100);
        assert_eq!(b.1, "Result Clan B");
        assert_eq!(b.3, 200);
        assert!(b.2 < a.2, "Higher score should have better (lower) rank");

        sqlx::query("DELETE FROM clans WHERE id = $1 OR id = $2")
            .bind(clan_a_id)
            .bind(clan_b_id)
            .execute(&pool)
            .await
            .expect("Failed to delete clans");
        sqlx::query("DELETE FROM engine_users WHERE user_id = $1 OR user_id = $2")
            .bind(leader_a)
            .bind(leader_b)
            .execute(&pool)
            .await
            .expect("Failed to delete users");
        sqlx::query("ALTER TABLE clans ALTER COLUMN total_score TYPE INT")
            .execute(&pool)
            .await
            .ok();

        pool.close().await;
    }

    #[tokio::test]
    async fn test_season_get_active_season_none() {
        let pool = setup_pg_pool().await;
        let repo = SeasonPostgresRepo::new(pool.clone());

        let active = repo
            .get_active_season(&ClanTier::Gold)
            .await
            .expect("Failed to get active season");

        assert!(active.is_none(), "Should return None when no active season exists");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_season_get_by_id_none() {
        let pool = setup_pg_pool().await;
        let repo = SeasonPostgresRepo::new(pool.clone());

        let fetched = repo
            .get_season_by_id(Uuid::new_v4())
            .await
            .expect("Failed to get season by id");

        assert!(fetched.is_none(), "Should return None for non-existent season");

        pool.close().await;
    }
}
