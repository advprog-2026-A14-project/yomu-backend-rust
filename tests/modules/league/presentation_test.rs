use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_api_create_clan_route_exists() {
    use yomu_backend_rust::modules::league::presentation::routes::league_routes;

    let state = yomu_backend_rust::AppState {
        db: sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect("postgres://yomu:yomu_password@localhost:5432/yomu_engine_test")
            .await
            .unwrap(),
        redis: redis::Client::open("redis://localhost:6379")
            .unwrap()
            .get_multiplexed_async_connection()
            .await
            .unwrap(),
    };

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let leader_id = Uuid::new_v4();
    let request_body = serde_json::json!({
        "name": "Test Clan",
        "leader_id": leader_id
    });

    let request = Request::builder()
        .uri("/api/v1/clans")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_api_join_clan_route_exists() {
    use yomu_backend_rust::modules::league::presentation::routes::league_routes;

    let state = yomu_backend_rust::AppState {
        db: sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect("postgres://yomu:yomu_password@localhost:5432/yomu_engine_test")
            .await
            .unwrap(),
        redis: redis::Client::open("redis://localhost:6379")
            .unwrap()
            .get_multiplexed_async_connection()
            .await
            .unwrap(),
    };

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let clan_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let request_body = serde_json::json!({
        "clan_id": clan_id,
        "user_id": user_id
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}/join", clan_id))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_get_leaderboard_route_exists() {
    use yomu_backend_rust::modules::league::presentation::routes::league_routes;

    let state = yomu_backend_rust::AppState {
        db: sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect("postgres://yomu:yomu_password@localhost:5432/yomu_engine_test")
            .await
            .unwrap(),
        redis: redis::Client::open("redis://localhost:6379")
            .unwrap()
            .get_multiplexed_async_connection()
            .await
            .unwrap(),
    };

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request = Request::builder()
        .uri("/api/v1/leaderboards?tier=Diamond")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
