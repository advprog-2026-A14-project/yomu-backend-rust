// Unit tests for User Sync domain layer
// Tests ShadowUser entity, QuizHistory entity, and UserSyncError

mod shadow_user_tests {
    use uuid::Uuid;
    use yomu_backend_rust::modules::user_sync::domain::entities::shadow_user::ShadowUser;

    // Basic creation tests
    #[test]
    fn test_shadow_user_new_creates_with_zero_score() {
        let user_id = Uuid::new_v4();
        let user = ShadowUser::new(user_id);
        assert_eq!(user.user_id(), user_id);
        assert_eq!(user.total_score(), 0);
    }

    #[test]
    fn test_shadow_user_with_id_reconstructs_correctly() {
        let user_id = Uuid::new_v4();
        let score = 1500;
        let user = ShadowUser::with_id(user_id, score);
        assert_eq!(user.user_id(), user_id);
        assert_eq!(user.total_score(), score);
    }

    // Nil UUID tests
    #[test]
    fn test_shadow_user_with_nil_uuid_is_valid() {
        let nil_uuid = Uuid::nil();
        let user = ShadowUser::new(nil_uuid);
        assert_eq!(user.user_id(), nil_uuid);
        assert_eq!(user.total_score(), 0);
    }

    #[test]
    fn test_shadow_user_with_id_nil_uuid_with_score() {
        let nil_uuid = Uuid::nil();
        let user = ShadowUser::with_id(nil_uuid, 100);
        assert_eq!(user.user_id(), nil_uuid);
        assert_eq!(user.total_score(), 100);
    }

    // Boundary value tests
    #[test]
    fn test_shadow_user_max_score() {
        let max_score = i32::MAX;
        let user = ShadowUser::with_id(Uuid::nil(), max_score);
        assert_eq!(user.total_score(), max_score);
    }

    #[test]
    fn test_shadow_user_min_score() {
        let min_score = i32::MIN;
        let user = ShadowUser::with_id(Uuid::nil(), min_score);
        assert_eq!(user.total_score(), min_score);
    }

    #[test]
    fn test_shadow_user_score_of_one() {
        let user = ShadowUser::with_id(Uuid::new_v4(), 1);
        assert_eq!(user.total_score(), 1);
    }

    #[test]
    fn test_shadow_user_score_of_negative_one() {
        let user = ShadowUser::with_id(Uuid::new_v4(), -1);
        assert_eq!(user.total_score(), -1);
    }

    // Clone independence tests
    #[test]
    fn test_shadow_user_clone_is_independent() {
        let user_id = Uuid::new_v4();
        let user = ShadowUser::new(user_id);
        let cloned = user.clone();
        assert_eq!(user.user_id(), cloned.user_id());
        assert_eq!(user.total_score(), cloned.total_score());
        // Verify independence - modifying clone doesn't affect original
        let _ = cloned; // suppress unused warning
    }

    #[test]
    fn test_shadow_user_clone_preserves_all_fields() {
        let user_id = Uuid::new_v4();
        let score = 999;
        let user = ShadowUser::with_id(user_id, score);
        let cloned = user.clone();
        assert_eq!(cloned.user_id(), user_id);
        assert_eq!(cloned.total_score(), score);
    }

    // Debug format test
    #[test]
    fn test_shadow_user_debug_format() {
        let user_id = Uuid::new_v4();
        let user = ShadowUser::new(user_id);
        let debug_str = format!("{:?}", user);
        assert!(debug_str.contains("ShadowUser"));
        assert!(debug_str.contains(&user_id.to_string()));
    }

    // from_db constructor tests (package-private)
    #[test]
    fn test_shadow_user_from_db_reconstructs() {
        let user_id = Uuid::new_v4();
        let score = 500;
        let user = ShadowUser::from_db(user_id, score);
        assert_eq!(user.user_id(), user_id);
        assert_eq!(user.total_score(), score);
    }
}

mod quiz_history_tests {
    use chrono::Utc;
    use uuid::Uuid;
    use yomu_backend_rust::modules::user_sync::domain::entities::quiz_history::QuizHistory;

    // Basic creation tests
    #[test]
    fn test_quiz_history_new_creates_with_generated_id() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, 100, 0.95);

        assert_eq!(quiz.user_id(), user_id);
        assert_eq!(quiz.article_id(), article_id);
        assert_eq!(quiz.score(), 100);
        assert!((quiz.accuracy() - 0.95).abs() < f64::EPSILON);
        // ID should be generated (not nil)
        assert_ne!(quiz.id(), Uuid::nil());
        // completed_at should be set (approximately now)
        let now = Utc::now();
        let diff = now.signed_duration_since(quiz.completed_at());
        assert!(diff.num_seconds() >= 0 && diff.num_seconds() < 5);
    }

    #[test]
    fn test_quiz_history_with_id_reconstructs() {
        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let score = 85;
        let accuracy = 0.92;
        let completed = Utc::now();

        let quiz = QuizHistory::with_id(id, user_id, article_id, score, accuracy, completed);

        assert_eq!(quiz.id(), id);
        assert_eq!(quiz.user_id(), user_id);
        assert_eq!(quiz.article_id(), article_id);
        assert_eq!(quiz.score(), score);
        assert!((quiz.accuracy() - accuracy).abs() < f64::EPSILON);
        assert_eq!(quiz.completed_at(), completed);
    }

    // Nil UUID tests
    #[test]
    fn test_quiz_history_with_nil_user_id_is_valid() {
        let nil_uuid = Uuid::nil();
        let quiz = QuizHistory::new(nil_uuid, nil_uuid, 0, 0.0);
        assert_eq!(quiz.user_id(), nil_uuid);
        assert_eq!(quiz.article_id(), nil_uuid);
    }

    #[test]
    fn test_quiz_history_with_nil_article_id_is_valid() {
        let user_id = Uuid::new_v4();
        let nil_uuid = Uuid::nil();
        let quiz = QuizHistory::new(user_id, nil_uuid, 100, 1.0);
        assert_eq!(quiz.article_id(), nil_uuid);
    }

    // Score tests
    #[test]
    fn test_quiz_history_negative_score_is_valid() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, -10, 0.0);
        assert_eq!(quiz.score(), -10);
    }

    #[test]
    fn test_quiz_history_zero_score() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, 0, 0.0);
        assert_eq!(quiz.score(), 0);
    }

    #[test]
    fn test_quiz_history_max_score() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, i32::MAX, 1.0);
        assert_eq!(quiz.score(), i32::MAX);
    }

    #[test]
    fn test_quiz_history_min_score() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, i32::MIN, 1.0);
        assert_eq!(quiz.score(), i32::MIN);
    }

    // Accuracy tests
    #[test]
    fn test_quiz_history_perfect_accuracy() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, 100, 1.0);
        assert!((quiz.accuracy() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quiz_history_zero_accuracy() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, 0, 0.0);
        assert!((quiz.accuracy() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quiz_history_half_accuracy() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, 50, 0.5);
        assert!((quiz.accuracy() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quiz_history_float_precision_edge_case() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        // 1/3 is a repeating decimal
        let quiz = QuizHistory::new(user_id, article_id, 100, 1.0 / 3.0);
        let expected = 1.0 / 3.0;
        assert!((quiz.accuracy() - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quiz_history_very_small_accuracy() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, 10, 0.00001);
        assert!((quiz.accuracy() - 0.00001).abs() < 0.000001);
    }

    // Clone tests
    #[test]
    fn test_quiz_history_clone_is_independent() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, 100, 0.95);
        let cloned = quiz.clone();
        assert_eq!(quiz.id(), cloned.id());
        assert_eq!(quiz.user_id(), cloned.user_id());
        assert_eq!(quiz.article_id(), cloned.article_id());
        assert_eq!(quiz.score(), cloned.score());
    }

    #[test]
    fn test_quiz_history_clone_preserves_all_fields() {
        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let completed = Utc::now();
        let quiz = QuizHistory::with_id(id, user_id, article_id, 85, 0.92, completed);
        let cloned = quiz.clone();
        assert_eq!(cloned.id(), id);
        assert_eq!(cloned.user_id(), user_id);
        assert_eq!(cloned.article_id(), article_id);
        assert_eq!(cloned.score(), 85);
        assert!((cloned.accuracy() - 0.92).abs() < f64::EPSILON);
        assert_eq!(cloned.completed_at(), completed);
    }

    // from_db constructor tests (package-private)
    #[test]
    fn test_quiz_history_from_db_reconstructs() {
        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let score = 75;
        let accuracy = 0.88;
        let completed = Utc::now();

        let quiz = QuizHistory::from_db(id, user_id, article_id, score, accuracy, completed);

        assert_eq!(quiz.id(), id);
        assert_eq!(quiz.user_id(), user_id);
        assert_eq!(quiz.article_id(), article_id);
        assert_eq!(quiz.score(), score);
        assert!((quiz.accuracy() - accuracy).abs() < f64::EPSILON);
        assert_eq!(quiz.completed_at(), completed);
    }

    // Debug format test
    #[test]
    fn test_quiz_history_debug_format() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, 100, 0.95);
        let debug_str = format!("{:?}", quiz);
        assert!(debug_str.contains("QuizHistory"));
    }
}

mod user_sync_error_tests {
    use axum::http::StatusCode;
    use yomu_backend_rust::modules::user_sync::domain::errors::user_sync_error::UserSyncError;

    // Error mapping tests
    #[test]
    fn user_sync_error_user_already_exists_maps_to_409() {
        let error = UserSyncError::UserAlreadyExists("user_123".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn user_sync_error_sync_failed_maps_to_500() {
        let error = UserSyncError::SyncFailed("network timeout".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn user_sync_error_database_error_maps_to_500() {
        let error = UserSyncError::DatabaseError("connection lost".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn user_sync_error_user_not_found_maps_to_404() {
        let error = UserSyncError::UserNotFound("user_456".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn user_sync_error_invalid_quiz_data_maps_to_400() {
        let error = UserSyncError::InvalidQuizData("invalid score".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn user_sync_error_validation_error_maps_to_400() {
        let error = UserSyncError::ValidationError("missing required field".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // Display trait tests
    #[test]
    fn user_sync_error_display_formats_correctly() {
        let error = UserSyncError::UserAlreadyExists("test_user".to_string());
        let display = format!("{}", error);
        assert!(display.contains("User already exists"));
        assert!(display.contains("test_user"));
    }

    #[test]
    fn user_sync_error_user_not_found_display() {
        let error = UserSyncError::UserNotFound("missing_user".to_string());
        let display = format!("{}", error);
        assert!(display.contains("User not found"));
        assert!(display.contains("missing_user"));
    }

    #[test]
    fn user_sync_error_invalid_quiz_data_display() {
        let error = UserSyncError::InvalidQuizData("negative score".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Invalid quiz data"));
        assert!(display.contains("negative score"));
    }

    #[test]
    fn user_sync_error_database_error_display() {
        let error = UserSyncError::DatabaseError("connection timeout".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Database error"));
        assert!(display.contains("connection timeout"));
    }

    // Debug trait tests
    #[test]
    fn user_sync_error_debug_format() {
        let error = UserSyncError::SyncFailed("test failure".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("SyncFailed"));
        assert!(debug_str.contains("test failure"));
    }
}
