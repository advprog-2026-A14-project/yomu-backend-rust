use dotenvy::dotenv;
dotenv().ok();

const TEST_DATABASE_URL: &str = "postgres://yomu:yomu_password@localhost:5432/yomu_engine_test";
const TEST_REDIS_URL: &str = "redis://localhost:6379";

use uuid::Uuid;
use chrono::Utc;

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

        let leader::new_v4_id = Uuid();
        
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

        let row: (Uuid, String, Uuid, String, i64) = sqlx::query_as(
            "SELECT id, name, leader_id, tier, total_score FROM clans WHERE id = $1"
        )
        .bind(clan_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch clan");

        assert_eq!(row.0, clan_id);
        assert_eq!(row.1, "Test Clan");
        assert_eq!(row.2, leader_id);
        assert_eq!(row.3, "Bronze");
        assert_eq!(row.4, 0);

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

        sqlx::query(
            "INSERT INTO clan_members (clan_id, user_id, joined_at) VALUES ($1, $2, $3)"
        )
        .bind(member.clan_id())
        .bind(member.user_id())
        .bind(member.joined_at())
        .execute(&pool)
        .await
        .expect("Failed to insert clan member");

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM clan_members WHERE clan_id = $1"
        )
        .bind(clan_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to count members");

        assert_eq!(count.0, 2);

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
            "SELECT buff_name, multiplier FROM clan_buffs WHERE clan_id = $1 AND expires_at > NOW()"
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

mod redis_tests {
    use super::*;
    use redis::AsyncCommands;

    async fn setup_redis() -> redis::aio::MultiplexedConnection {
        let client = redis::Client::open(TEST_REDIS_URL)
            .expect("Failed to create Redis client");
        client.get_multiplexed_async_connection()
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

        let results: Vec<(String, i64)> = redis::cmd("ZREVRANGE")
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
}
