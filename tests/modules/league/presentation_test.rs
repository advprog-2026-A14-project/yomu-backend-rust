use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

const TEST_DATABASE_URL: &str = "postgres://yomu:yomu_password@localhost:5432/yomu_engine_test";

async fn setup_test_user(pool: &sqlx::PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

async fn setup_test_clan(
    pool: &sqlx::PgPool,
    clan_id: Uuid,
    leader_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, NOW())"
    )
    .bind(clan_id)
    .bind("Test Clan")
    .bind(leader_id)
    .bind("Bronze")
    .bind(0i32)
    .execute(pool)
    .await?;
    Ok(())
}

async fn cleanup_test_data(
    pool: &sqlx::PgPool,
    clan_id: Uuid,
    leader_id: Uuid,
    member_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    if let Some(mid) = member_id {
        let _ = sqlx::query("DELETE FROM clan_members WHERE clan_id = $1 AND user_id = $2")
            .bind(clan_id)
            .bind(mid)
            .execute(pool)
            .await;
    }
    let _ = sqlx::query("DELETE FROM clans WHERE id = $1")
        .bind(clan_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
        .bind(leader_id)
        .execute(pool)
        .await;
    if let Some(mid) = member_id {
        let _ = sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
            .bind(mid)
            .execute(pool)
            .await;
    }
    Ok(())
}

#[tokio::test]
async fn test_api_create_clan_route_exists() {
    use yomu_backend_rust::modules::league::presentation::routes::league_routes;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(TEST_DATABASE_URL)
        .await
        .unwrap();

    let state = yomu_backend_rust::AppState {
        db: pool.clone(),
        redis: redis::Client::open("redis://localhost:6379")
            .unwrap()
            .get_multiplexed_async_connection()
            .await
            .unwrap(),
    };

    let leader_id = Uuid::new_v4();

    // Setup: create leader user first
    setup_test_user(&pool, leader_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

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

    // Cleanup
    let _ = sqlx::query("DELETE FROM clan_members WHERE user_id = $1")
        .bind(leader_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM clans WHERE leader_id = $1")
        .bind(leader_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
        .bind(leader_id)
        .execute(&pool)
        .await;
    pool.close().await;

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_api_join_clan_route_exists() {
    use yomu_backend_rust::modules::league::presentation::routes::league_routes;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(TEST_DATABASE_URL)
        .await
        .unwrap();

    let state = yomu_backend_rust::AppState {
        db: pool.clone(),
        redis: redis::Client::open("redis://localhost:6379")
            .unwrap()
            .get_multiplexed_async_connection()
            .await
            .unwrap(),
    };

    let leader_id = Uuid::new_v4();
    let member_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();

    // Setup: create leader, member, and clan first
    setup_test_user(&pool, leader_id).await.unwrap();
    setup_test_user(&pool, member_id).await.unwrap();
    setup_test_clan(&pool, clan_id, leader_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request_body = serde_json::json!({
        "clan_id": clan_id,
        "user_id": member_id
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}/join", clan_id))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
    eprintln!("Response status: {}, body: {}", status, body_str);

    // Cleanup
    cleanup_test_data(&pool, clan_id, leader_id, Some(member_id))
        .await
        .unwrap();
    pool.close().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_api_get_leaderboard_route_exists() {
    use yomu_backend_rust::modules::league::presentation::routes::league_routes;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(TEST_DATABASE_URL)
        .await
        .unwrap();

    let state = yomu_backend_rust::AppState {
        db: pool.clone(),
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
    pool.close().await;

    assert_eq!(response.status(), StatusCode::OK);
}
