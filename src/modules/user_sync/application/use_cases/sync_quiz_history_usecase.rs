use crate::modules::user_sync::application::dto::QuizHistoryRequestDto;
use crate::modules::user_sync::application::dto::QuizHistoryResponseDto;
use crate::modules::user_sync::domain::entities::quiz_history::QuizHistory;
use crate::modules::user_sync::domain::errors::UserSyncError;
use crate::modules::user_sync::domain::repositories::QuizHistoryRepository;
use crate::modules::user_sync::domain::repositories::UserRepository;

pub struct SyncQuizHistoryUseCase<U: UserRepository, Q: QuizHistoryRepository> {
    user_repo: U,
    quiz_repo: Q,
}

impl<U: UserRepository, Q: QuizHistoryRepository> SyncQuizHistoryUseCase<U, Q> {
    pub fn new(user_repo: U, quiz_repo: Q) -> Self {
        Self {
            user_repo,
            quiz_repo,
        }
    }

    pub async fn execute(
        &self,
        dto: QuizHistoryRequestDto,
    ) -> Result<QuizHistoryResponseDto, UserSyncError> {
        if dto.score < 0 {
            return Err(UserSyncError::InvalidQuizData(format!(
                "Score tidak boleh negatif: {}",
                dto.score
            )));
        }

        if dto.accuracy < 0.0 || dto.accuracy > 100.0 {
            return Err(UserSyncError::InvalidQuizData(format!(
                "Accuracy harus antara 0.0 dan 100.0, diterima: {}",
                dto.accuracy
            )));
        }

        let exists = self
            .user_repo
            .exists_shadow_user(dto.user_id)
            .await
            .map_err(|e| UserSyncError::DatabaseError(e.to_string()))?;

        if !exists {
            return Err(UserSyncError::UserNotFound(format!(
                "User ID {} tidak ditemukan di Engine DB, harap sync user terlebih dahulu",
                dto.user_id
            )));
        }

        let quiz = QuizHistory::new(dto.user_id, dto.article_id, dto.score, dto.accuracy);

        self.quiz_repo
            .insert_quiz_history(&quiz)
            .await
            .map_err(|e| UserSyncError::DatabaseError(e.to_string()))?;

        self.user_repo
            .update_total_score(dto.user_id, dto.score)
            .await
            .map_err(|e| UserSyncError::DatabaseError(e.to_string()))?;

        Ok(QuizHistoryResponseDto {
            user_id: dto.user_id,
            missions_updated: 0,
            message: "Data riwayat kuis berhasil dicatat dan diproses oleh Engine".to_string(),
        })
    }
}
