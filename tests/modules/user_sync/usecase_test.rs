// Unit tests for User Sync use cases
// Tests SyncNewUserUseCase and SyncQuizHistoryUseCase with mocked repositories

use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;
use yomu_backend_rust::modules::user_sync::application::SyncNewUserUseCase;
use yomu_backend_rust::modules::user_sync::application::SyncQuizHistoryUseCase;
use yomu_backend_rust::modules::user_sync::application::dto::QuizHistoryRequestDto;
use yomu_backend_rust::modules::user_sync::application::dto::SyncUserRequestDto;
use yomu_backend_rust::modules::user_sync::domain::entities::quiz_history::QuizHistory;
use yomu_backend_rust::modules::user_sync::domain::entities::shadow_user::ShadowUser;
use yomu_backend_rust::modules::user_sync::domain::repositories::QuizHistoryRepository;
use yomu_backend_rust::modules::user_sync::domain::repositories::UserRepository;
use yomu_backend_rust::shared::domain::base_error::AppError;

// Mock definitions for UserRepository
mock! {
    UserRepo {}

    #[async_trait]
    impl UserRepository for UserRepo {
        async fn insert_shadow_user(&self, user: &ShadowUser) -> Result<(), AppError>;
        async fn exists_shadow_user(&self, user_id: Uuid) -> Result<bool, AppError>;
        async fn check_exists(&self, user_id: Uuid) -> bool;
        async fn get_shadow_user(&self, user_id: Uuid) -> Result<Option<ShadowUser>, AppError>;
        async fn update_total_score(&self, user_id: Uuid, score_to_add: i32) -> Result<(), AppError>;
    }
}

// Mock definitions for QuizHistoryRepository
mock! {
    QuizRepo {}

    #[async_trait]
    impl QuizHistoryRepository for QuizRepo {
        async fn insert_quiz_history(&self, quiz: &QuizHistory) -> Result<(), AppError>;
        async fn get_quiz_histories_by_user(&self, user_id: Uuid, limit: Option<i64>) -> Result<Vec<QuizHistory>, AppError>;
    }
}

// ============== SyncNewUserUseCase Tests ==============

mod sync_new_user_tests {
    use super::*;

    #[tokio::test]
    async fn sync_new_user_success() {
        let user_id = Uuid::new_v4();
        let mut mock_repo = MockUserRepo::new();

        mock_repo
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(false))
            .once();

        mock_repo
            .expect_insert_shadow_user()
            .return_once(|_| Ok(()))
            .once();

        let use_case = SyncNewUserUseCase::new(mock_repo);
        let dto = SyncUserRequestDto { user_id };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.user_id(), user_id);
        assert_eq!(user.total_score(), 0);
    }

    #[tokio::test]
    async fn sync_new_user_already_exists() {
        let user_id = Uuid::new_v4();
        let mut mock_repo = MockUserRepo::new();

        mock_repo
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        mock_repo
            .expect_get_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(move |_| Ok(Some(ShadowUser::with_id(user_id, 0))))
            .once();

        let use_case = SyncNewUserUseCase::new(mock_repo);
        let dto = SyncUserRequestDto { user_id };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().user_id(), user_id);
    }

    #[tokio::test]
    async fn sync_new_user_nil_uuid() {
        let nil_uuid = Uuid::nil();
        let mut mock_repo = MockUserRepo::new();

        mock_repo
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(nil_uuid))
            .return_once(|_| Ok(false))
            .once();

        mock_repo
            .expect_insert_shadow_user()
            .return_once(|_| Ok(()))
            .once();

        let use_case = SyncNewUserUseCase::new(mock_repo);
        let dto = SyncUserRequestDto { user_id: nil_uuid };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_new_user_db_error_on_exists() {
        let user_id = Uuid::new_v4();
        let mut mock_repo = MockUserRepo::new();

        mock_repo
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Err(AppError::InternalServer("DB error".to_string())))
            .once();

        let use_case = SyncNewUserUseCase::new(mock_repo);
        let dto = SyncUserRequestDto { user_id };

        let result = use_case.execute(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_new_user_db_error_on_insert() {
        let user_id = Uuid::new_v4();
        let mut mock_repo = MockUserRepo::new();

        mock_repo
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(false))
            .once();

        mock_repo
            .expect_insert_shadow_user()
            .return_once(|_| Err(AppError::InternalServer("Insert failed".to_string())))
            .once();

        let use_case = SyncNewUserUseCase::new(mock_repo);
        let dto = SyncUserRequestDto { user_id };

        let result = use_case.execute(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_new_user_max_uuid() {
        // Test with max UUID value
        let max_uuid = Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();
        let mut mock_repo = MockUserRepo::new();

        mock_repo
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(max_uuid))
            .return_once(|_| Ok(false))
            .once();

        mock_repo
            .expect_insert_shadow_user()
            .return_once(|_| Ok(()))
            .once();

        let use_case = SyncNewUserUseCase::new(mock_repo);
        let dto = SyncUserRequestDto { user_id: max_uuid };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }
}

// ============== SyncQuizHistoryUseCase Tests ==============

mod sync_quiz_history_tests {
    use super::*;

    #[tokio::test]
    async fn sync_quiz_history_success() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(100))
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 100,
            accuracy: 0.95,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.user_id, user_id);
    }

    #[tokio::test]
    async fn sync_quiz_history_user_not_found() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(false))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 100,
            accuracy: 0.95,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_quiz_history_negative_score_rejected() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let user_mock = MockUserRepo::new();
        let quiz_mock = MockQuizRepo::new();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: -10,
            accuracy: 0.95,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_quiz_history_accuracy_below_zero_rejected() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let user_mock = MockUserRepo::new();
        let quiz_mock = MockQuizRepo::new();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 100,
            accuracy: -0.1,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_quiz_history_accuracy_above_100_rejected() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let user_mock = MockUserRepo::new();
        let quiz_mock = MockQuizRepo::new();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 100,
            accuracy: 100.1,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_quiz_history_zero_score() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(0))
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 0,
            accuracy: 0.0,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_history_perfect_accuracy() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(100))
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 100,
            accuracy: 1.0,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_history_db_error_on_insert() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Err(AppError::InternalServer("Insert failed".to_string())))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 100,
            accuracy: 0.95,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_quiz_history_db_error_on_update_score() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(100))
            .return_once(|_, _| Err(AppError::InternalServer("Update failed".to_string())))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 100,
            accuracy: 0.95,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_quiz_history_exists_check_db_error() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Err(AppError::InternalServer("DB error".to_string())))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 100,
            accuracy: 0.95,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_quiz_history_nil_user_id() {
        let nil_uuid = Uuid::nil();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(nil_uuid))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(nil_uuid), mockall::predicate::eq(50))
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id: nil_uuid,
            article_id,
            score: 50,
            accuracy: 0.8,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_history_nil_article_id() {
        let user_id = Uuid::new_v4();
        let nil_uuid = Uuid::nil();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(75))
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id: nil_uuid,
            score: 75,
            accuracy: 0.9,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_history_max_score() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(i32::MAX),
            )
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: i32::MAX,
            accuracy: 1.0,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_history_boundary_accuracy_0() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(0))
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 0,
            accuracy: 0.0,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_history_boundary_accuracy_100() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(100))
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 100,
            accuracy: 100.0,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_history_zero_accuracy_edge_case() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(10))
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 10,
            accuracy: 0.0000001,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sync_quiz_history_response_contains_user_id() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let mut user_mock = MockUserRepo::new();
        let mut quiz_mock = MockQuizRepo::new();

        user_mock
            .expect_exists_shadow_user()
            .with(mockall::predicate::eq(user_id))
            .return_once(|_| Ok(true))
            .once();

        quiz_mock
            .expect_insert_quiz_history()
            .return_once(|_| Ok(()))
            .once();

        user_mock
            .expect_update_total_score()
            .with(mockall::predicate::eq(user_id), mockall::predicate::eq(85))
            .return_once(|_, _| Ok(()))
            .once();

        let use_case = SyncQuizHistoryUseCase::new(user_mock, quiz_mock);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: 85,
            accuracy: 0.92,
        };

        let result = use_case.execute(dto).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.user_id, user_id);
        assert!(response.missions_updated >= 0);
    }
}
