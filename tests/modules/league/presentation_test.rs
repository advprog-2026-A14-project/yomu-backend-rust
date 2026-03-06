use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use mockall::mock;
use tower::ServiceExt;
use uuid::Uuid;

use yomu_backend_rust::modules::league::application::dto::LeaderboardEntry;
use yomu_backend_rust::modules::league::application::dto::{
    CreateClanDto, JoinClanDto, UpdateScoreDto,
};
use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
use yomu_backend_rust::modules::league::domain::entities::clan_member::ClanMember;
use yomu_backend_rust::modules::league::domain::repositories::ClanRepository;
use yomu_backend_rust::modules::league::domain::repositories::LeaderboardCache;
use yomu_backend_rust::shared::domain::base_error::AppError;

mock! {
    ClanRepositoryRepo {}
    #[async_trait]
    impl ClanRepository for ClanRepositoryRepo {
        async fn create_clan(&self, clan: &Clan) -> Result<(), AppError>;
        async fn get_clan_by_id(&self, clan_id: Uuid) -> Result<Option<Clan>, AppError>;
        async fn add_member(&self, member: &ClanMember) -> Result<(), AppError>;
        async fn is_user_in_any_clan(&self, user_id: Uuid) -> Result<bool, AppError>;
        async fn get_user_clan_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError>;
        async fn add_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
    }
}

mock! {
    LeaderboardCacheRepo {}
    #[async_trait]
    impl LeaderboardCache for LeaderboardCacheRepo {
        async fn update_clan_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError>;
        async fn get_top_clans(&self, tier: &str, limit: usize) -> Result<Vec<LeaderboardEntry>, AppError>;
    }
}

fn create_test_router() -> axum::Router {
    axum::Router::new()
}

#[tokio::test]
async fn test_api_create_clan() {
    let app = create_test_router();
    let leader_id = Uuid::new_v4();
    let request_body = CreateClanDto {
        name: "Test Clan".to_string(),
        leader_id,
    };

    let request = Request::builder()
        .uri("/api/v1/clans")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Expected 404 Not Found, got {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_api_join_clan() {
    let app = create_test_router();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let request_body = JoinClanDto { clan_id, user_id };

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}/join", clan_id))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Expected 404 Not Found, got {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_api_get_leaderboard() {
    let app = create_test_router();

    let request = Request::builder()
        .uri("/api/v1/leaderboards?tier=Diamond")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Expected 404 Not Found, got {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_api_update_score() {
    let app = create_test_router();
    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let request_body = UpdateScoreDto {
        clan_id,
        user_id,
        base_score: 100,
        multiplier: 1.5,
    };

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}/score", clan_id))
        .method("PUT")
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&request_body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Expected 404 Not Found, got {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_api_get_clan() {
    let app = create_test_router();
    let clan_id = Uuid::new_v4();

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}", clan_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NOT_FOUND,
        "Expected 404 Not Found, got {:?}",
        response.status()
    );
}
