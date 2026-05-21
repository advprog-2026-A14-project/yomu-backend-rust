use tonic::{Request, Response, Status};

use crate::generated::health::{
    HealthCheckRequest, HealthCheckResponse, health_service_server::HealthService,
};

pub struct GrpcHealthHandler;

impl GrpcHealthHandler {
    pub fn new() -> Self {
        Self
    }
}

#[tonic::async_trait]
impl HealthService for GrpcHealthHandler {
    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: "SERVING".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }
}

impl Default for GrpcHealthHandler {
    fn default() -> Self {
        Self::new()
    }
}
