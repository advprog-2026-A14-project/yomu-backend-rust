use tracing::instrument;
use tonic::{Request, Response, Status};

use crate::generated::quizsync::{
    SyncQuizHistoryRequest, SyncQuizHistoryResponse, quiz_sync_service_server::QuizSyncService,
};
use crate::modules::user_sync::application::dto::QuizHistoryRequestDto;
use crate::modules::user_sync::application::use_cases::sync_quiz_history_usecase::SyncQuizHistoryUseCase;
use crate::modules::user_sync::infrastructure::database::postgres::quiz_history_postgres_repo::QuizHistoryPostgresRepo;
use crate::modules::user_sync::infrastructure::database::postgres::user_postgres_repo::UserPostgresRepo;
use sqlx::PgPool;

#[derive(Debug)]
pub struct QuizSyncGrpcHandler {
    db: PgPool,
}

impl QuizSyncGrpcHandler {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl QuizSyncService for QuizSyncGrpcHandler {
    #[instrument]
    async fn sync_quiz_history(
        &self,
        request: Request<SyncQuizHistoryRequest>,
    ) -> Result<Response<SyncQuizHistoryResponse>, Status> {
        let req = request.into_inner();
        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("invalid user_id"))?;
        let article_id = uuid::Uuid::parse_str(&req.article_id)
            .map_err(|_| Status::invalid_argument("invalid article_id"))?;

        let user_repo = UserPostgresRepo::new(self.db.clone());
        let quiz_repo = QuizHistoryPostgresRepo::new(self.db.clone());
        let use_case = SyncQuizHistoryUseCase::new(user_repo, quiz_repo);
        let dto = QuizHistoryRequestDto {
            user_id,
            article_id,
            score: req.score as i32,
            accuracy: req.accuracy,
        };

        let result = use_case.execute(dto).await;

        match result {
            Ok(response) => Ok(Response::new(SyncQuizHistoryResponse {
                user_id: req.user_id,
                missions_updated: response.missions_updated,
                message: response.message,
            })),
            Err(crate::modules::user_sync::domain::errors::UserSyncError::UserNotFound(_)) => {
                Err(Status::not_found("user not found"))
            }
            Err(crate::modules::user_sync::domain::errors::UserSyncError::InvalidQuizData(_)) => {
                Err(Status::invalid_argument("invalid quiz data"))
            }
            Err(_) => Err(Status::internal("sync failed")),
        }
    }
}
