use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QuizHistory {
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    id: Uuid,
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    user_id: Uuid,
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    article_id: Uuid,
    #[schema(value_type = i32, example = 85)]
    score: i32,
    #[schema(value_type = f64, example = 0.92)]
    accuracy: f64,
    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    completed_at: DateTime<Utc>,
}

impl QuizHistory {
    pub fn new(user_id: Uuid, article_id: Uuid, score: i32, accuracy: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            article_id,
            score,
            accuracy,
            completed_at: Utc::now(),
        }
    }

    #[allow(dead_code)]
    pub fn with_id(
        id: Uuid,
        user_id: Uuid,
        article_id: Uuid,
        score: i32,
        accuracy: f64,
        completed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            article_id,
            score,
            accuracy,
            completed_at,
        }
    }

    /// Package-private constructor for reconstructing from database
    pub(crate) fn from_db(
        id: Uuid,
        user_id: Uuid,
        article_id: Uuid,
        score: i32,
        accuracy: f64,
        completed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            user_id,
            article_id,
            score,
            accuracy,
            completed_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn article_id(&self) -> Uuid {
        self.article_id
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn accuracy(&self) -> f64 {
        self.accuracy
    }

    pub fn completed_at(&self) -> DateTime<Utc> {
        self.completed_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quiz_history_new_creates_with_generated_id() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, 100, 0.95);

        assert_eq!(quiz.user_id(), user_id);
        assert_eq!(quiz.article_id(), article_id);
        assert_eq!(quiz.score(), 100);
        assert!((quiz.accuracy() - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quiz_history_with_nil_uuid_is_valid() {
        let nil_uuid = Uuid::nil();
        let quiz = QuizHistory::new(nil_uuid, nil_uuid, 0, 0.0);

        assert_eq!(quiz.user_id(), nil_uuid);
        assert_eq!(quiz.article_id(), nil_uuid);
    }

    #[test]
    fn test_quiz_history_negative_score_is_valid() {
        let user_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();
        let quiz = QuizHistory::new(user_id, article_id, -10, 0.0);

        assert_eq!(quiz.score(), -10);
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
}
