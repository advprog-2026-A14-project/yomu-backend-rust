use crate::modules::user_sync::domain::entities::quiz_history::QuizHistory;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait QuizHistoryRepository: Send + Sync {
    async fn insert_quiz_history(&self, quiz: &QuizHistory) -> Result<(), AppError>;
    async fn get_quiz_histories_by_user(&self, user_id: Uuid)
    -> Result<Vec<QuizHistory>, AppError>;
}
