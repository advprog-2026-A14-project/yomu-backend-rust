const TEST_DATABASE_URL: &str = "postgres://yomu:yomu_password@localhost:5432/yomu_engine_test";

mod pg_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use uuid::Uuid;

    async fn setup_pg_pool() -> sqlx::PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(TEST_DATABASE_URL)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_pg_insert_and_get_achievement() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind("Test Achievement")
        .bind(5i32)
        .bind("Common")
        .bind("QuizComplete")
        .bind(100i32)
        .execute(&pool)
        .await
        .expect("Failed to insert achievement");

        let row: (Uuid, String, i32, String, String, i32) = sqlx::query_as(
            "SELECT id, name, milestone_target, achievement_type, trigger_type, reward_points \
             FROM achievements WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch achievement");

        assert_eq!(row.0, id);
        assert_eq!(row.1, "Test Achievement");
        assert_eq!(row.2, 5);
        assert_eq!(row.3, "Common");
        assert_eq!(row.4, "QuizComplete");
        assert_eq!(row.5, 100);

        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Failed to clean up achievement");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_achievement_not_found_returns_none() {
        let pool = setup_pg_pool().await;
        let nonexistent_id = Uuid::new_v4();

        let result: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM achievements WHERE id = $1")
            .bind(nonexistent_id)
            .fetch_optional(&pool)
            .await
            .expect("Query failed");

        assert!(result.is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_user_achievement_insert_and_query() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        sqlx::query(
            "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(achievement_id)
        .bind("Progress Achievement")
        .bind(10i32)
        .bind("Rare")
        .bind("ReadArticle")
        .bind(200i32)
        .execute(&pool)
        .await
        .expect("Failed to insert achievement");

        sqlx::query(
            "INSERT INTO user_achievements \
             (user_id, achievement_id, current_progress, is_completed, is_shown_on_profile) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(user_id)
        .bind(achievement_id)
        .bind(4i32)
        .bind(false)
        .bind(false)
        .execute(&pool)
        .await
        .expect("Failed to insert user_achievement");

        let row: (i32, bool, bool) = sqlx::query_as(
            "SELECT current_progress, is_completed, is_shown_on_profile \
             FROM user_achievements WHERE user_id = $1 AND achievement_id = $2",
        )
        .bind(user_id)
        .bind(achievement_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user_achievement");

        assert_eq!(row.0, 4);
        assert!(!row.1);
        assert!(!row.2);

        sqlx::query("DELETE FROM user_achievements WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(achievement_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_user_achievement_upsert_updates_progress() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        sqlx::query(
            "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(achievement_id)
        .bind("Upsert Achievement")
        .bind(5i32)
        .bind("Epic")
        .bind("QuizComplete")
        .bind(300i32)
        .execute(&pool)
        .await
        .expect("Failed to insert achievement");

        sqlx::query(
            "INSERT INTO user_achievements \
             (user_id, achievement_id, current_progress, is_completed, is_shown_on_profile) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(user_id)
        .bind(achievement_id)
        .bind(2i32)
        .bind(false)
        .bind(false)
        .execute(&pool)
        .await
        .expect("Failed to insert initial user_achievement");

        sqlx::query(
            "INSERT INTO user_achievements \
             (user_id, achievement_id, current_progress, is_completed, is_shown_on_profile) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (user_id, achievement_id) DO UPDATE SET \
                 current_progress = EXCLUDED.current_progress, \
                 is_completed = EXCLUDED.is_completed",
        )
        .bind(user_id)
        .bind(achievement_id)
        .bind(5i32)
        .bind(true)
        .bind(false)
        .execute(&pool)
        .await
        .expect("Failed to upsert user_achievement");

        let row: (i32, bool) = sqlx::query_as(
            "SELECT current_progress, is_completed FROM user_achievements \
             WHERE user_id = $1 AND achievement_id = $2",
        )
        .bind(user_id)
        .bind(achievement_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch updated user_achievement");

        assert_eq!(row.0, 5);
        assert!(row.1);

        sqlx::query("DELETE FROM user_achievements WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(achievement_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_user_score_accumulates_via_upsert() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        sqlx::query(
            "INSERT INTO engine_users (user_id, total_score) VALUES ($1, $2) \
             ON CONFLICT (user_id) DO UPDATE SET \
                 total_score = engine_users.total_score + EXCLUDED.total_score",
        )
        .bind(user_id)
        .bind(100i32)
        .execute(&pool)
        .await
        .expect("Failed to add first score");

        sqlx::query(
            "INSERT INTO engine_users (user_id, total_score) VALUES ($1, $2) \
             ON CONFLICT (user_id) DO UPDATE SET \
                 total_score = engine_users.total_score + EXCLUDED.total_score",
        )
        .bind(user_id)
        .bind(50i32)
        .execute(&pool)
        .await
        .expect("Failed to add second score");

        let score: (i32,) =
            sqlx::query_as("SELECT total_score FROM engine_users WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to fetch score");

        assert_eq!(score.0, 150);

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_multiple_achievements_for_user() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let ach_id_1 = Uuid::new_v4();
        let ach_id_2 = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        for (id, name) in [(ach_id_1, "First"), (ach_id_2, "Second")] {
            sqlx::query(
                "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(id)
            .bind(name)
            .bind(3i32)
            .bind("Common")
            .bind("QuizComplete")
            .bind(50i32)
            .execute(&pool)
            .await
            .expect("Failed to insert achievement");

            sqlx::query(
                "INSERT INTO user_achievements \
                 (user_id, achievement_id, current_progress, is_completed, is_shown_on_profile) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(user_id)
            .bind(id)
            .bind(1i32)
            .bind(false)
            .bind(false)
            .execute(&pool)
            .await
            .expect("Failed to insert user_achievement");
        }

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_achievements WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to count user_achievements");

        assert_eq!(count.0, 2);

        sqlx::query("DELETE FROM user_achievements WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        for id in [ach_id_1, ach_id_2] {
            sqlx::query("DELETE FROM achievements WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .expect("Cleanup failed");
        }

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }
}
