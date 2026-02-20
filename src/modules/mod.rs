// Modules (Bounded Contexts) - League, Gamification, User Sync
use axum::Router;
use sqlx::PgPool;

pub mod gamification;

pub fn all_routes() -> Router<PgPool> {
    Router::new()
        .merge(gamification::routes())
}