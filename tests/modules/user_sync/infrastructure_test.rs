// Integration tests for User Sync PostgreSQL repository
const TEST_DATABASE_URL: &str = "postgres://yomu:yomu_password@localhost:5432/yomu_engine_test";

use uuid::Uuid;
use yomu_backend_rust::modules::user_sync::domain::entities::shadow_user::ShadowUser;
use yomu_backend_rust::modules::user_sync::domain::entities::quiz_history::QuizHistory;
use yomu_backend_rust::modules::user_sync::domain::repositories::UserRepository;
use yomu_backend_rust::modules::user_sync::domain::repositories::quiz_history_repository::QuizHistoryRepository;
use yomu_backend_rust::modules::user_sync::infrastructure::database::postgres::user_postgres_repo::UserPostgresRepo;
use yomu_backend_rust::modules::user_sync::infrastructure::database::postgres::quiz_history_postgres_repo::QuizHistoryPostgresRepo;

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

    async fn cleanup_all_test_data(pool: &sqlx::PgPool, user_id: Uuid) {
        let _ = sqlx::query("DELETE FROM quiz_history WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM shadow_users WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await;
    }

    #[tokio::test]
    async fn test_pg_insert_shadow_user() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let repo = UserPostgresRepo::new(pool.clone());
        let user = ShadowUser::new(user_id);

        let result = repo.insert_shadow_user(&user).await;
        assert!(result.is_ok(), "Insert should succeed");

        let exists = repo
            .exists_shadow_user(user_id)
            .await
            .expect("Query should succeed");
        assert!(exists, "User should exist after insert");

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_exists_shadow_user_returns_true_for_existing() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let repo = UserPostgresRepo::new(pool.clone());
        let user = ShadowUser::new(user_id);
        repo.insert_shadow_user(&user)
            .await
            .expect("Insert should succeed");

        let exists = repo
            .exists_shadow_user(user_id)
            .await
            .expect("Query should succeed");
        assert!(exists, "Should return true for existing user");

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_exists_shadow_user_returns_false_for_nonexistent() {
        let pool = setup_pg_pool().await;
        let non_existent_id = Uuid::new_v4();

        let repo = UserPostgresRepo::new(pool.clone());

        let exists = repo
            .exists_shadow_user(non_existent_id)
            .await
            .expect("Query should succeed");
        assert!(!exists, "Should return false for non-existent user");

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_get_shadow_user_returns_user_when_exists() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let repo = UserPostgresRepo::new(pool.clone());
        let user = ShadowUser::new(user_id);
        repo.insert_shadow_user(&user)
            .await
            .expect("Insert should succeed");

        let retrieved = repo
            .get_shadow_user(user_id)
            .await
            .expect("Query should succeed");
        assert!(retrieved.is_some(), "Should retrieve user");
        assert_eq!(retrieved.unwrap().user_id(), user_id);

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_get_shadow_user_returns_none_for_nonexistent() {
        let pool = setup_pg_pool().await;
        let non_existent_id = Uuid::new_v4();

        let repo = UserPostgresRepo::new(pool.clone());

        let retrieved = repo
            .get_shadow_user(non_existent_id)
            .await
            .expect("Query should succeed");
        assert!(
            retrieved.is_none(),
            "Should return None for non-existent user"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_update_total_score_increments_score() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let repo = UserPostgresRepo::new(pool.clone());
        let user = ShadowUser::new(user_id);
        repo.insert_shadow_user(&user)
            .await
            .expect("Insert should succeed");

        repo.update_total_score(user_id, 100)
            .await
            .expect("Update should succeed");

        let retrieved = repo
            .get_shadow_user(user_id)
            .await
            .expect("Query should succeed");
        let retrieved_user = retrieved.unwrap();
        assert_eq!(
            retrieved_user.total_score(),
            100,
            "Score should be incremented to 100"
        );

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_update_total_score_accumulates() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let repo = UserPostgresRepo::new(pool.clone());
        let user = ShadowUser::new(user_id);
        repo.insert_shadow_user(&user)
            .await
            .expect("Insert should succeed");

        repo.update_total_score(user_id, 50)
            .await
            .expect("First update should succeed");
        repo.update_total_score(user_id, 30)
            .await
            .expect("Second update should succeed");
        repo.update_total_score(user_id, 20)
            .await
            .expect("Third update should succeed");

        let retrieved = repo
            .get_shadow_user(user_id)
            .await
            .expect("Query should succeed");
        assert_eq!(
            retrieved.unwrap().total_score(),
            100,
            "Score should accumulate to 100"
        );

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_insert_duplicate_shadow_user_does_not_error() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let repo = UserPostgresRepo::new(pool.clone());
        let user = ShadowUser::new(user_id);

        let result1 = repo.insert_shadow_user(&user).await;
        assert!(result1.is_ok(), "First insert should succeed");

        let result2 = repo.insert_shadow_user(&user).await;
        assert!(
            result2.is_ok(),
            "Duplicate insert should not error due to ON CONFLICT DO NOTHING"
        );

        let exists = repo
            .exists_shadow_user(user_id)
            .await
            .expect("Query should succeed");
        assert!(exists, "User should still exist");

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_insert_quiz_history() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let repo = QuizHistoryPostgresRepo::new(pool.clone());
        let quiz = QuizHistory::new(user_id, article_id, 85, 0.92);

        let result = repo.insert_quiz_history(&quiz).await;
        assert!(result.is_ok(), "Insert quiz history should succeed");

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_get_quiz_histories_by_user_returns_histories() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let article_id_1 = Uuid::new_v4();
        let article_id_2 = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let repo = QuizHistoryPostgresRepo::new(pool.clone());

        let quiz1 = QuizHistory::new(user_id, article_id_1, 100, 1.0);
        let quiz2 = QuizHistory::new(user_id, article_id_2, 80, 0.85);

        repo.insert_quiz_history(&quiz1)
            .await
            .expect("First insert should succeed");
        repo.insert_quiz_history(&quiz2)
            .await
            .expect("Second insert should succeed");

        let histories = repo
            .get_quiz_histories_by_user(user_id)
            .await
            .expect("Query should succeed");
        assert_eq!(histories.len(), 2, "Should return 2 quiz histories");

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_get_quiz_histories_by_user_returns_empty_for_nonexistent() {
        let pool = setup_pg_pool().await;
        let non_existent_id = Uuid::new_v4();

        let repo = QuizHistoryPostgresRepo::new(pool.clone());

        let histories = repo
            .get_quiz_histories_by_user(non_existent_id)
            .await
            .expect("Query should succeed");
        assert!(
            histories.is_empty(),
            "Should return empty vec for non-existent user"
        );

        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_insert_quiz_history_updates_shadow_user_score() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let user_repo = UserPostgresRepo::new(pool.clone());
        let shadow_user = ShadowUser::new(user_id);
        user_repo
            .insert_shadow_user(&shadow_user)
            .await
            .expect("Insert shadow user should succeed");

        let quiz_repo = QuizHistoryPostgresRepo::new(pool.clone());
        let quiz = QuizHistory::new(user_id, article_id, 75, 0.88);
        quiz_repo
            .insert_quiz_history(&quiz)
            .await
            .expect("Insert quiz history should succeed");

        user_repo
            .update_total_score(user_id, 75)
            .await
            .expect("Update score should succeed");

        let retrieved = user_repo
            .get_shadow_user(user_id)
            .await
            .expect("Query should succeed");
        assert_eq!(
            retrieved.unwrap().total_score(),
            75,
            "Shadow user score should be updated"
        );

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_check_exists_returns_true_for_existing_user() {
        let pool = setup_pg_pool().await;
        let user_id = Uuid::new_v4();

        sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Failed to insert engine user");

        let repo = UserPostgresRepo::new(pool.clone());
        let user = ShadowUser::new(user_id);
        repo.insert_shadow_user(&user)
            .await
            .expect("Insert should succeed");

        let exists = repo.check_exists(user_id).await;
        assert!(exists, "check_exists should return true for existing user");

        cleanup_all_test_data(&pool, user_id).await;
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pg_check_exists_returns_false_for_nonexistent_user() {
        let pool = setup_pg_pool().await;
        let non_existent_id = Uuid::new_v4();

        let repo = UserPostgresRepo::new(pool.clone());

        let exists = repo.check_exists(non_existent_id).await;
        assert!(
            !exists,
            "check_exists should return false for non-existent user"
        );

        pool.close().await;
    }
}
