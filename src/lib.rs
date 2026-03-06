// Library root - exports all modules for testing and external use

pub mod config;
pub mod modules;
pub mod shared;

// Re-export AppState for use in modules
pub use config::database::{init_postgres_pool, init_redis_pool};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
pub use shared::domain::base_error::AppError;
pub use shared::utils::response::ApiResponse;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: PgPool,
    pub redis: MultiplexedConnection,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
