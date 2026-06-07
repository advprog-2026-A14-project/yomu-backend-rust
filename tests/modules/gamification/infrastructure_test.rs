const TEST_DATABASE_URL: &str = "postgres://yomu:yomu_password@localhost:5432/yomu_engine_test";

mod pg_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use uuid::Uuid;
    use yomu_backend_rust::modules::gamification::infrastructure::database::postgres::achievement_repository::PostgresAchievementRepository;
    use yomu_backend_rust::modules::gamification::infrastructure::database::postgres::mission_repository::PostgresMissionRepository;
    use yomu_backend_rust::modules::gamification::domain::repositories::achievement_repository::AchievementRepository;
    use yomu_backend_rust::modules::gamification::domain::repositories::mission_repository::MissionRepository;
    use yomu_backend_rust::modules::gamification::domain::entities::achievement::{Achievement, AchievementTriggerType, AchievementType};
    use yomu_backend_rust::modules::gamification::domain::entities::user_achievement::UserAchievement;
    use yomu_backend_rust::modules::gamification::domain::entities::daily_mission::{DailyMission, MissionType};
    use yomu_backend_rust::modules::gamification::domain::entities::user_mission::UserMission;
    use chrono::{Utc, NaiveDate};

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

    #[tokio::test]
    async fn test_achievement_repo_get_by_id_found() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();
        let repo = PostgresAchievementRepository { pool: pool.clone() };

        sqlx::query(
            "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
             VALUES ($1, $2, $3, $4, $5, $6)"
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

        let result = repo.get_achievement_by_id(id).await;
        assert!(result.is_ok());
        let achievement = result.unwrap();
        assert!(achievement.is_some());
        let a = achievement.unwrap();
        assert_eq!(a.id(), id);
        assert_eq!(a.name(), "Test Achievement");
        assert_eq!(a.milestone_target(), 5);
        assert_eq!(a.reward_points(), 100);

        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Failed to clean up achievement");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_achievement_repo_get_by_id_not_found() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();
        let repo = PostgresAchievementRepository { pool: pool.clone() };

        let result = repo.get_achievement_by_id(id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_achievement_repo_get_all_achievements() {
        let pool = setup_pg_pool().await;
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let repo = PostgresAchievementRepository { pool: pool.clone() };

        for (id, name) in [(id1, "First"), (id2, "Second")] {
            sqlx::query(
                "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
                 VALUES ($1, $2, $3, $4, $5, $6)"
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
        }

        let result = repo.get_all_achievements().await;
        assert!(result.is_ok());
        let achievements = result.unwrap();
        let ids: Vec<Uuid> = achievements.iter().map(|a| a.id()).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));

        for id in [id1, id2] {
            sqlx::query("DELETE FROM achievements WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .expect("Cleanup failed");
        }

        pool.close().await;
    }

    #[tokio::test]
    async fn test_achievement_repo_get_by_ids() {
        let pool = setup_pg_pool().await;
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let repo = PostgresAchievementRepository { pool: pool.clone() };

        for (id, name) in [(id1, "First"), (id2, "Second"), (id3, "Third")] {
            sqlx::query(
                "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
                 VALUES ($1, $2, $3, $4, $5, $6)"
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
        }

        let result = repo.get_achievements_by_ids(&[id1, id2]).await;
        assert!(result.is_ok());
        let achievements = result.unwrap();
        assert_eq!(achievements.len(), 2);
        let ids: Vec<Uuid> = achievements.iter().map(|a| a.id()).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));

        for id in [id1, id2, id3] {
            sqlx::query("DELETE FROM achievements WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .expect("Cleanup failed");
        }

        pool.close().await;
    }

    #[tokio::test]
    async fn test_achievement_repo_get_user_achievements() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let ach_id = Uuid::new_v4();
        let repo = PostgresAchievementRepository { pool: pool.clone() };

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        sqlx::query(
            "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(ach_id)
        .bind("Progress Achievement")
        .bind(10i32)
        .bind("Rare")
        .bind("ReadArticle")
        .bind(200i32)
        .execute(&pool)
        .await
        .expect("Failed to insert achievement");

        sqlx::query(
            "INSERT INTO user_achievements (user_id, achievement_id, current_progress, is_completed, is_shown_on_profile) \
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(user_id)
        .bind(ach_id)
        .bind(4i32)
        .bind(false)
        .bind(false)
        .execute(&pool)
        .await
        .expect("Failed to insert user_achievement");

        let result = repo.get_user_achievements(user_id).await;
        assert!(result.is_ok());
        let uas = result.unwrap();
        assert_eq!(uas.len(), 1);
        assert_eq!(uas[0].achievement_id(), ach_id);
        assert_eq!(uas[0].current_progress(), 4);

        sqlx::query("DELETE FROM user_achievements WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(ach_id)
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
    async fn test_achievement_repo_get_user_achievement() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let ach_id = Uuid::new_v4();
        let repo = PostgresAchievementRepository { pool: pool.clone() };

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        sqlx::query(
            "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(ach_id)
        .bind("Single Achievement")
        .bind(5i32)
        .bind("Epic")
        .bind("QuizComplete")
        .bind(300i32)
        .execute(&pool)
        .await
        .expect("Failed to insert achievement");

        sqlx::query(
            "INSERT INTO user_achievements (user_id, achievement_id, current_progress, is_completed, is_shown_on_profile) \
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(user_id)
        .bind(ach_id)
        .bind(5i32)
        .bind(true)
        .bind(false)
        .execute(&pool)
        .await
        .expect("Failed to insert user_achievement");

        let result = repo.get_user_achievement(user_id, ach_id).await;
        assert!(result.is_ok());
        let ua = result.unwrap();
        assert!(ua.is_some());
        let u = ua.unwrap();
        assert_eq!(u.achievement_id(), ach_id);
        assert!(u.is_completed());

        sqlx::query("DELETE FROM user_achievements WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(ach_id)
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
    async fn test_achievement_repo_save_user_achievement() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let ach_id = Uuid::new_v4();
        let repo = PostgresAchievementRepository { pool: pool.clone() };

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        sqlx::query(
            "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(ach_id)
        .bind("Save Achievement")
        .bind(5i32)
        .bind("Common")
        .bind("QuizComplete")
        .bind(50i32)
        .execute(&pool)
        .await
        .expect("Failed to insert achievement");

        let mut ua = UserAchievement::new(user_id, ach_id);
        ua.current_progress = 3;
        let result = repo.save_user_achievement(&ua).await;
        assert!(result.is_ok());

        let row: (i32,) = sqlx::query_as(
            "SELECT current_progress FROM user_achievements WHERE user_id = $1 AND achievement_id = $2"
        )
        .bind(user_id)
        .bind(ach_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user_achievement");
        assert_eq!(row.0, 3);

        sqlx::query("DELETE FROM user_achievements WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(ach_id)
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
    async fn test_achievement_repo_add_user_score() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let repo = PostgresAchievementRepository { pool: pool.clone() };

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, $2)")
            .bind(user_id)
            .bind(20i32)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        let result = repo.add_user_score(user_id, 30).await;
        assert!(result.is_ok());

        let score: (i32,) = sqlx::query_as("SELECT total_score FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch score");
        assert_eq!(score.0, 50);

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_achievement_repo_create_achievement() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();
        let repo = PostgresAchievementRepository { pool: pool.clone() };

        let achievement = Achievement::new(
            id,
            "Created Achievement".to_string(),
            10,
            AchievementType::Rare,
            AchievementTriggerType::ReadArticle,
            250,
        ).expect("Failed to create achievement");

        let result = repo.create_achievement(&achievement).await;
        assert!(result.is_ok());

        let row: (String, i32, String, String, i32) = sqlx::query_as(
            "SELECT name, milestone_target, achievement_type, trigger_type, reward_points FROM achievements WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch achievement");
        assert_eq!(row.0, "Created Achievement");
        assert_eq!(row.1, 10);
        assert_eq!(row.2, "Rare");
        assert_eq!(row.3, "ReadArticle");
        assert_eq!(row.4, 250);

        sqlx::query("DELETE FROM achievements WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_create_daily_mission() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();

        let mission = DailyMission::new(
            id,
            "Read 5 articles".to_string(),
            5,
            date,
            100,
            MissionType::ReadArticle,
        ).expect("Failed to create daily mission");

        let result = repo.create_daily_mission(&mission).await;
        assert!(result.is_ok());

        let row: (String, i32, NaiveDate, i32, String) = sqlx::query_as(
            "SELECT description, target_count, date, reward_points, mission_type FROM daily_missions WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch mission");
        assert_eq!(row.0, "Read 5 articles");
        assert_eq!(row.1, 5);
        assert_eq!(row.2, date);
        assert_eq!(row.3, 100);
        assert_eq!(row.4, "ReadArticle");

        sqlx::query("DELETE FROM daily_missions WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_get_active_missions_by_date() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };
        let today = Utc::now().naive_utc().date();

        sqlx::query(
            "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind("Today Mission")
        .bind(3i32)
        .bind(today)
        .bind(50i32)
        .bind("Quiz")
        .execute(&pool)
        .await
        .expect("Failed to insert mission");

        let result = repo.get_active_missions_by_date(today).await;
        assert!(result.is_ok());
        let missions = result.unwrap();
        let ids: Vec<Uuid> = missions.iter().map(|m| m.id()).collect();
        assert!(ids.contains(&id));

        sqlx::query("DELETE FROM daily_missions WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_get_user_mission() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let mission_id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        sqlx::query(
            "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(mission_id)
        .bind("Mission")
        .bind(2i32)
        .bind(NaiveDate::from_ymd_opt(2026, 6, 7).unwrap())
        .bind(20i32)
        .bind("Quiz")
        .execute(&pool)
        .await
        .expect("Failed to insert mission");

        sqlx::query(
            "INSERT INTO user_missions (user_id, mission_id, current_progress, is_claimed) \
             VALUES ($1, $2, $3, $4)"
        )
        .bind(user_id)
        .bind(mission_id)
        .bind(1i32)
        .bind(false)
        .execute(&pool)
        .await
        .expect("Failed to insert user_mission");

        let result = repo.get_user_mission(user_id, mission_id).await;
        assert!(result.is_ok());
        let um = result.unwrap();
        assert!(um.is_some());
        let u = um.unwrap();
        assert_eq!(u.mission_id(), mission_id);
        assert_eq!(u.current_progress(), 1);

        sqlx::query("DELETE FROM user_missions WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        sqlx::query("DELETE FROM daily_missions WHERE id = $1")
            .bind(mission_id)
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
    async fn test_mission_repo_get_user_missions_batch() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let mid1 = Uuid::new_v4();
        let mid2 = Uuid::new_v4();
        let mid3 = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        for mid in [mid1, mid2, mid3] {
            sqlx::query(
                "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
                 VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind(mid)
            .bind("Mission")
            .bind(2i32)
            .bind(NaiveDate::from_ymd_opt(2026, 6, 7).unwrap())
            .bind(20i32)
            .bind("Quiz")
            .execute(&pool)
            .await
            .expect("Failed to insert mission");

            sqlx::query(
                "INSERT INTO user_missions (user_id, mission_id, current_progress, is_claimed) \
                 VALUES ($1, $2, $3, $4)"
            )
            .bind(user_id)
            .bind(mid)
            .bind(1i32)
            .bind(false)
            .execute(&pool)
            .await
            .expect("Failed to insert user_mission");
        }

        let result = repo.get_user_missions_batch(user_id, vec![mid1, mid2]).await;
        assert!(result.is_ok());
        let ums = result.unwrap();
        assert_eq!(ums.len(), 2);
        let ids: Vec<Uuid> = ums.iter().map(|u| u.mission_id()).collect();
        assert!(ids.contains(&mid1));
        assert!(ids.contains(&mid2));

        sqlx::query("DELETE FROM user_missions WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        for mid in [mid1, mid2, mid3] {
            sqlx::query("DELETE FROM daily_missions WHERE id = $1")
                .bind(mid)
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

    #[tokio::test]
    async fn test_mission_repo_save_user_mission() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let mission_id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        sqlx::query(
            "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(mission_id)
        .bind("Mission")
        .bind(2i32)
        .bind(NaiveDate::from_ymd_opt(2026, 6, 7).unwrap())
        .bind(20i32)
        .bind("Quiz")
        .execute(&pool)
        .await
        .expect("Failed to insert mission");

        let mut um = UserMission::new(user_id, mission_id);
        um.current_progress = 2;
        let result = repo.save_user_mission(&um).await;
        assert!(result.is_ok());

        let row: (i32, bool) = sqlx::query_as(
            "SELECT current_progress, is_claimed FROM user_missions WHERE user_id = $1 AND mission_id = $2"
        )
        .bind(user_id)
        .bind(mission_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user_mission");
        assert_eq!(row.0, 2);
        assert!(!row.1);

        sqlx::query("DELETE FROM user_missions WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        sqlx::query("DELETE FROM daily_missions WHERE id = $1")
            .bind(mission_id)
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
    async fn test_mission_repo_update_daily_mission() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();

        sqlx::query(
            "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind("Old Description")
        .bind(3i32)
        .bind(date)
        .bind(50i32)
        .bind("Quiz")
        .execute(&pool)
        .await
        .expect("Failed to insert mission");

        let updated = DailyMission::new(
            id,
            "Updated Description".to_string(),
            5,
            date,
            75,
            MissionType::DailyLogin,
        ).expect("Failed to create updated mission");

        let result = repo.update_daily_mission(&updated).await;
        assert!(result.is_ok());

        let row: (String, i32, i32, String) = sqlx::query_as(
            "SELECT description, target_count, reward_points, mission_type FROM daily_missions WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch mission");
        assert_eq!(row.0, "Updated Description");
        assert_eq!(row.1, 5);
        assert_eq!(row.2, 75);
        assert_eq!(row.3, "DailyLogin");

        sqlx::query("DELETE FROM daily_missions WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_delete_daily_mission() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();

        sqlx::query(
            "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind("Delete Mission")
        .bind(3i32)
        .bind(date)
        .bind(50i32)
        .bind("Quiz")
        .execute(&pool)
        .await
        .expect("Failed to insert mission");

        let result = repo.delete_daily_mission(id).await;
        assert!(result.is_ok());

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM daily_missions WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count");
        assert_eq!(count.0, 0);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_add_user_score() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, $2)")
            .bind(user_id)
            .bind(15i32)
            .execute(&pool)
            .await
            .expect("Failed to insert engine_user");

        let result = repo.add_user_score(user_id, 25).await;
        assert!(result.is_ok());

        let score: (i32,) = sqlx::query_as("SELECT total_score FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch score");
        assert_eq!(score.0, 40);

        sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_get_active_missions_by_date_multiple() {
        let pool = setup_pg_pool().await;
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();

        for (id, desc) in [(id1, "Mission One"), (id2, "Mission Two")] {
            sqlx::query(
                "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
                 VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind(id)
            .bind(desc)
            .bind(2i32)
            .bind(date)
            .bind(20i32)
            .bind("Quiz")
            .execute(&pool)
            .await
            .expect("Failed to insert mission");
        }

        let result = repo.get_active_missions_by_date(date).await;
        assert!(result.is_ok());
        let missions = result.unwrap();
        let ids: Vec<Uuid> = missions.iter().map(|m| m.id()).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));

        for id in [id1, id2] {
            sqlx::query("DELETE FROM daily_missions WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .expect("Cleanup failed");
        }

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_get_user_missions_empty_batch() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };

        let result = repo.get_user_missions_batch(user_id, vec![]).await;
        assert!(result.is_ok());
        let ums = result.unwrap();
        assert!(ums.is_empty());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_update_daily_mission_not_found() {
        let pool = setup_pg_pool().await;
        let repo = PostgresMissionRepository { pool: pool.clone() };
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();

        let updated = DailyMission::new(
            Uuid::new_v4(),
            "Does not exist".to_string(),
            5,
            date,
            75,
            MissionType::DailyLogin,
        ).expect("Failed to create mission");

        let result = repo.update_daily_mission(&updated).await;
        assert!(result.is_err());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_delete_daily_mission_not_found() {
        let pool = setup_pg_pool().await;
        let repo = PostgresMissionRepository { pool: pool.clone() };

        let result = repo.delete_daily_mission(Uuid::new_v4()).await;
        assert!(result.is_err());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_get_daily_mission_by_id_found() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();

        sqlx::query(
            "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind("Found Mission")
        .bind(3i32)
        .bind(date)
        .bind(50i32)
        .bind("Quiz")
        .execute(&pool)
        .await
        .expect("Failed to insert mission");

        let result = repo.get_daily_mission_by_id(id).await;
        assert!(result.is_ok());
        let mission = result.unwrap();
        assert!(mission.is_some());
        let m = mission.unwrap();
        assert_eq!(m.id(), id);
        assert_eq!(m.description(), "Found Mission");
        assert_eq!(m.target_count(), 3);
        assert_eq!(m.reward_points(), 50);

        sqlx::query("DELETE FROM daily_missions WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_get_daily_mission_by_id_not_found() {
        let pool = setup_pg_pool().await;
        let repo = PostgresMissionRepository { pool: pool.clone() };

        let result = repo.get_daily_mission_by_id(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        pool.close().await;
    }

    #[tokio::test]
    async fn test_mission_repo_get_daily_mission_by_id_invalid_data() {
        let pool = setup_pg_pool().await;
        let id = Uuid::new_v4();
        let repo = PostgresMissionRepository { pool: pool.clone() };
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();

        sqlx::query(
            "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind("")
        .bind(3i32)
        .bind(date)
        .bind(50i32)
        .bind("Quiz")
        .execute(&pool)
        .await
        .expect("Failed to insert mission");

        let result = repo.get_daily_mission_by_id(id).await;
        assert!(result.is_err());

        sqlx::query("DELETE FROM daily_missions WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Cleanup failed");

        pool.close().await;
    }
}
