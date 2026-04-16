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
use utoipa::OpenApi;

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

#[derive(OpenApi)]
#[openapi(
    paths(
        modules::league::presentation::controllers::clan_controller::create_clan_handler,
        modules::league::presentation::controllers::clan_controller::join_clan_handler,
        modules::league::presentation::controllers::score_controller::get_leaderboard_handler,
        modules::user_sync::presentation::controllers::internal_user_controller::sync_user_handler,
    ),
    components(
        schemas(
            modules::league::application::dto::CreateClanDto,
            modules::league::application::dto::JoinClanDto,
            modules::league::application::dto::LeaderboardDto,
            modules::league::application::dto::LeaderboardEntry,
            modules::league::domain::entities::clan::Clan,
            modules::league::domain::entities::clan::ClanTier,
            modules::league::domain::entities::clan_member::ClanMember,
            modules::league::domain::entities::clan_member::MemberRole,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "clans", description = "Clan management endpoints"),
        (name = "leaderboard", description = "Leaderboard endpoints"),
        (name = "users", description = "User synchronization endpoints")
    ),
    info(
        title = "Yomu Backend Rust API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Gamification engine API for Yomu learning platform. Provides clan management, leaderboards, and user synchronization capabilities."
    )
)]
pub struct ApiDoc;
