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

        let key_a = format!("leaderboard:{}:{}", tier, clan_a_id);
        let key_b = format!("leaderboard:{}:{}", tier, clan_b_id);

        let _: () = redis::cmd("ZADD")
            .arg(&key_a)
            .arg(100)
            .arg(clan_a_id.to_string())
            .query_async(&mut con)
            .await
            .expect("Failed to add score for clan A");

        let _: () = redis::cmd("ZADD")
            .arg(&key_b)
            .arg(50)
            .arg(clan_b_id.to_string())
            .query_async(&mut con)
            .await
            .expect("Failed to add score for clan B");

        let _: Vec<(String, i64)> = redis::cmd("ZREVRANGE")
            .arg(&key_a)
            .arg(0)
            .arg(10)
            .arg("WITHSCORES")
            .query_async(&mut con)
            .await
            .expect("Failed to get top clans");

        let leaderboard_key = format!("leaderboard:{}", tier);

        let top_clans: Vec<(String, i64)> = redis::cmd("ZREVRANGE")
            .arg(&leaderboard_key)
            .arg(0)
            .arg(10)
            .arg("WITHSCORES")
            .query_async(&mut con)
            .await
            .unwrap_or_else(|_| vec![]);

        let _: () = redis::cmd("DEL")
            .arg(&key_a)
            .query_async(&mut con)
            .await
            .expect("Failed to clean up key A");

        let _: () = redis::cmd("DEL")
            .arg(&key_b)
            .query_async(&mut con)
            .await
            .expect("Failed to clean up key B");

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
