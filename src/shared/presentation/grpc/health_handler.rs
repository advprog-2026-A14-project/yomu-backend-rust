use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

use crate::generated::health::{
    HealthCheckRequest, HealthCheckResponse, health_service_server::HealthService,
};

pub struct GrpcHealthHandler {
    db: PgPool,
    redis: MultiplexedConnection,
}

impl GrpcHealthHandler {
    pub fn new(db: PgPool, redis: MultiplexedConnection) -> Self {
        Self { db, redis }
    }
}

#[tonic::async_trait]
impl HealthService for GrpcHealthHandler {
    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let mut status = "SERVING".to_string();

        let postgres_ok = sqlx::query("SELECT 1").fetch_one(&self.db).await.is_ok();

        let mut redis_conn = self.redis.clone();
        let redis_ok = redis::cmd("PING")
            .query_async::<String>(&mut redis_conn)
            .await
            .is_ok_and(|resp| resp == "PONG");

        if !postgres_ok || !redis_ok {
            status = "NOT_SERVING".to_string();
        }

        Ok(Response::new(HealthCheckResponse {
            status,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }
}
