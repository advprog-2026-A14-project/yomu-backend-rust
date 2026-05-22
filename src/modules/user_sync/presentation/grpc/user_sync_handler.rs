use tracing::instrument;
use tonic::{Request, Response, Status};

use crate::generated::usersync::{
    SyncShadowUserRequest, SyncShadowUserResponse, user_sync_service_server::UserSyncService,
};
use crate::modules::user_sync::application::dto::SyncUserRequestDto;
use crate::modules::user_sync::application::use_cases::sync_new_user_usecase::SyncNewUserUseCase;
use crate::modules::user_sync::infrastructure::database::postgres::user_postgres_repo::UserPostgresRepo;
use sqlx::PgPool;

#[derive(Debug)]
pub struct UserSyncGrpcHandler {
    db: PgPool,
}

impl UserSyncGrpcHandler {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl UserSyncService for UserSyncGrpcHandler {
    #[instrument]
    async fn sync_shadow_user(
        &self,
        request: Request<SyncShadowUserRequest>,
    ) -> Result<Response<SyncShadowUserResponse>, Status> {
        let req = request.into_inner();
        let user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("invalid user_id"))?;

        let repo = UserPostgresRepo::new(self.db.clone());
        let use_case = SyncNewUserUseCase::new(repo);
        let dto = SyncUserRequestDto { user_id };

        let result = use_case.execute(dto).await;

        match result {
            Ok(_) => Ok(Response::new(SyncShadowUserResponse {
                user_id: req.user_id,
                message: "Shadow user berhasil disinkronisasi".to_string(),
            })),
            Err(_) => Err(Status::internal("sync failed")),
        }
    }
}
