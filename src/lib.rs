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
    pub postgres: String,
    pub redis: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        // League - Clans
        crate::modules::league::presentation::controllers::clan_controller::create_clan_handler,
        crate::modules::league::presentation::controllers::clan_controller::join_clan_handler,
        crate::modules::league::presentation::controllers::clan_controller::get_clan_detail_handler,
        crate::modules::league::presentation::controllers::clan_controller::get_user_tier_handler,
        // League - Leaderboard
        crate::modules::league::presentation::controllers::score_controller::get_leaderboard_handler,
    ),
    components(
        schemas(
            // League DTOs
            crate::modules::league::application::dto::CreateClanDto,
            crate::modules::league::application::dto::JoinClanDto,
            crate::modules::league::application::dto::LeaderboardDto,
            crate::modules::league::application::dto::LeaderboardEntry,
            crate::modules::league::application::dto::clan_detail_dto::ClanDetailDto,
            crate::modules::league::application::dto::clan_detail_dto::ClanMemberDto,
            crate::modules::league::application::dto::user_tier_dto::UserTierDto,
            // League Entities
            crate::modules::league::domain::entities::clan::Clan,
            crate::modules::league::domain::entities::clan::ClanTier,
            crate::modules::league::domain::entities::clan_member::ClanMember,
            crate::modules::league::domain::entities::clan_member::MemberRole,
        )
    ),
    tags(
        (name = "clans", description = "Clan management endpoints - create, join, and view clan details"),
        (name = "leaderboard", description = "Leaderboard endpoints for clan rankings by tier"),
        (name = "User Sync", description = "User synchronization endpoints from Java backend")
    ),
    info(
        title = "Yomu Backend Rust API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Gamification engine API for Yomu learning platform. Provides clan management, leaderboards, and user synchronization capabilities."
    )
)]
pub struct ApiDoc;
