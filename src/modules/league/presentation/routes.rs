use super::controllers::{clan_controller, score_controller};
use crate::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn league_routes() -> Router<AppState> {
    Router::new()
        .route("/clans", post(clan_controller::create_clan_handler))
        .route("/clans/{id}/join", post(clan_controller::join_clan_handler))
        .route(
            "/leaderboards",
            get(score_controller::get_leaderboard_handler),
        )
}
