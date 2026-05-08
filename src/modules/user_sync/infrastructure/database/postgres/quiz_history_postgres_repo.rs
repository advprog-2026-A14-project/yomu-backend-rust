use crate::modules::user_sync::domain::entities::quiz_history::QuizHistory;
use crate::modules::user_sync::domain::repositories::quiz_history_repository::QuizHistoryRepository;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct QuizHistoryRow {
    id: Uuid,
    user_id: Uuid,
    article_id: Uuid,
    score: i32,
    accuracy: f64,
    completed_at: DateTime<Utc>,
}

pub struct QuizHistoryPostgresRepo {
    pool: PgPool,
}

impl QuizHistoryPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QuizHistoryRepository for QuizHistoryPostgresRepo {
    async fn insert_quiz_history(&self, quiz: &QuizHistory) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO quiz_history (id, user_id, article_id, score, accuracy, completed_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(quiz.id())
        .bind(quiz.user_id())
        .bind(quiz.article_id())
        .bind(quiz.score())
        .bind(quiz.accuracy())
        .bind(quiz.completed_at())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(())
    }

    async fn get_quiz_histories_by_user(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<QuizHistory>, AppError> {
        let limit = limit.unwrap_or(100);
        let rows = sqlx::query_as::<_, QuizHistoryRow>(
            "SELECT id, user_id, article_id, score, CAST(accuracy AS FLOAT8) as accuracy, completed_at FROM quiz_history WHERE user_id = $1 ORDER BY completed_at DESC LIMIT $2"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                QuizHistory::from_db(
                    row.id,
                    row.user_id,
                    row.article_id,
                    row.score,
                    row.accuracy,
                    row.completed_at,
                )
            })
            .collect())
    }
}
