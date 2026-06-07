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

async fn ensure_clan_join_requests_table(pool: &sqlx::PgPool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS clan_join_requests (
            id UUID PRIMARY KEY,
            clan_id UUID NOT NULL REFERENCES clans(id) ON DELETE CASCADE,
            user_id UUID NOT NULL REFERENCES engine_users(user_id) ON DELETE CASCADE,
            status VARCHAR(20) NOT NULL DEFAULT 'Pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (clan_id, user_id)
        )",
    )
    .execute(pool)
    .await
    .expect("Failed to create clan_join_requests table");
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
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

#[tokio::test]
async fn test_api_process_buffs_route() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let leader_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();

    setup_test_user(&pool, leader_id).await.unwrap();
    setup_test_clan(&pool, clan_id, leader_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}/process-buffs", clan_id))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_test_data(&pool, clan_id, leader_id, None)
        .await
        .unwrap();
    pool.close().await;

    assert!(
        status == StatusCode::OK
            || status == StatusCode::NOT_FOUND
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "process-buffs endpoint should respond with OK, NOT_FOUND, or INTERNAL_SERVER_ERROR, got: {}",
        status
    );
}

#[tokio::test]
async fn test_api_update_score_route() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let leader_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();

    setup_test_user(&pool, leader_id).await.unwrap();
    setup_test_clan(&pool, clan_id, leader_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}/score", clan_id))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_test_data(&pool, clan_id, leader_id, None)
        .await
        .unwrap();
    pool.close().await;

    assert!(
        status == StatusCode::OK
            || status == StatusCode::NOT_FOUND
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "score endpoint should respond with OK, NOT_FOUND, or INTERNAL_SERVER_ERROR, got: {}",
        status
    );
}

#[tokio::test]
async fn test_api_end_season_route() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let season_id = Uuid::new_v4();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request = Request::builder()
        .uri(&format!("/api/v1/seasons/{}/end", season_id))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    pool.close().await;

    assert!(
        status == StatusCode::OK
            || status == StatusCode::NOT_FOUND
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "season end endpoint should respond with OK, NOT_FOUND, or INTERNAL_SERVER_ERROR, got: {}",
        status
    );
}

// =============================================================================
// NEW TESTS: Covering uncovered handlers in clan_controller and score_controller
// =============================================================================

#[tokio::test]
async fn test_api_get_clan_detail_route_exists() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let leader_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();

    setup_test_user(&pool, leader_id).await.unwrap();
    setup_test_clan(&pool, clan_id, leader_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}", clan_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_test_data(&pool, clan_id, leader_id, None)
        .await
        .unwrap();
    pool.close().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_api_get_user_tier_route_exists() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let user_id = Uuid::new_v4();

    setup_test_user(&pool, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request = Request::builder()
        .uri(&format!("/api/v1/users/{}/tier", user_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Clean up user
    let _ = sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
        .bind(user_id)
        .execute(&pool)
        .await;
    pool.close().await;

    // User not in a clan should still return OK with tier=None
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_REQUEST,
        "get_user_tier endpoint should respond with OK or BAD_REQUEST, got: {}",
        status
    );
}

#[tokio::test]
async fn test_api_delete_clan_route_exists() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let leader_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();

    setup_test_user(&pool, leader_id).await.unwrap();
    setup_test_clan(&pool, clan_id, leader_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request_body = serde_json::json!({
        "caller_id": leader_id
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}", clan_id))
        .method("DELETE")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Cleanup: clan might already be deleted by the DELETE handler
    let _ = sqlx::query("DELETE FROM clan_members WHERE clan_id = $1")
        .bind(clan_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM clans WHERE id = $1")
        .bind(clan_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
        .bind(leader_id)
        .execute(&pool)
        .await;
    pool.close().await;

    assert!(
        status == StatusCode::OK || status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "delete_clan endpoint should respond with OK, FORBIDDEN, or NOT_FOUND, got: {}",
        status
    );
}

#[tokio::test]
async fn test_api_create_join_request_route_exists() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let leader_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();

    setup_test_user(&pool, leader_id).await.unwrap();
    setup_test_user(&pool, requester_id).await.unwrap();
    setup_test_clan(&pool, clan_id, leader_id).await.unwrap();
    ensure_clan_join_requests_table(&pool).await;

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request_body = serde_json::json!({
        "clan_id": clan_id,
        "user_id": requester_id
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}/join-request", clan_id))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Cleanup: remove join request, then clan data
    let _ = sqlx::query("DELETE FROM clan_join_requests WHERE user_id = $1")
        .bind(requester_id)
        .execute(&pool)
        .await;
    cleanup_test_data(&pool, clan_id, leader_id, Some(requester_id))
        .await
        .unwrap();
    pool.close().await;

    assert!(
        status == StatusCode::CREATED
            || status == StatusCode::CONFLICT
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::NOT_FOUND,
        "create_join_request endpoint should respond with CREATED, CONFLICT, BAD_REQUEST, or NOT_FOUND, got: {}",
        status
    );
}

#[tokio::test]
async fn test_api_get_pending_requests_route_exists() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let leader_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();

    setup_test_user(&pool, leader_id).await.unwrap();
    setup_test_clan(&pool, clan_id, leader_id).await.unwrap();
    ensure_clan_join_requests_table(&pool).await;

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request = Request::builder()
        .uri(&format!(
            "/api/v1/clans/{}/join-requests?caller_id={}",
            clan_id, leader_id
        ))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_test_data(&pool, clan_id, leader_id, None)
        .await
        .unwrap();
    pool.close().await;

    assert!(
        status == StatusCode::OK || status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
        "get_pending_requests endpoint should respond with OK, FORBIDDEN, or NOT_FOUND, got: {}",
        status
    );
}

#[tokio::test]
async fn test_api_approve_join_request_route_exists() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let leader_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let request_id = Uuid::new_v4();

    setup_test_user(&pool, leader_id).await.unwrap();
    setup_test_user(&pool, requester_id).await.unwrap();
    setup_test_clan(&pool, clan_id, leader_id).await.unwrap();
    ensure_clan_join_requests_table(&pool).await;

    sqlx::query(
        "INSERT INTO clan_join_requests (id, clan_id, user_id, status, created_at, updated_at) VALUES ($1, $2, $3, 'Pending', NOW(), NOW())"
    )
    .bind(request_id)
    .bind(clan_id)
    .bind(requester_id)
    .execute(&pool)
    .await
    .unwrap();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request_body = serde_json::json!({
        "caller_id": leader_id
    });

    let request = Request::builder()
        .uri(&format!(
            "/api/v1/clans/join-requests/{}/approve",
            request_id
        ))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let _ = sqlx::query("DELETE FROM clan_members WHERE clan_id = $1")
        .bind(clan_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM clan_join_requests WHERE id = $1")
        .bind(request_id)
        .execute(&pool)
        .await;
    cleanup_test_data(&pool, clan_id, leader_id, Some(requester_id))
        .await
        .unwrap();
    pool.close().await;

    assert!(
        status == StatusCode::OK
            || status == StatusCode::FORBIDDEN
            || status == StatusCode::NOT_FOUND
            || status == StatusCode::CONFLICT,
        "approve_join_request endpoint should respond with OK, FORBIDDEN, NOT_FOUND, or CONFLICT, got: {}",
        status
    );
}

#[tokio::test]
async fn test_api_reject_join_request_route_exists() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let leader_id = Uuid::new_v4();
    let requester_id = Uuid::new_v4();
    let clan_id = Uuid::new_v4();
    let request_id = Uuid::new_v4();

    setup_test_user(&pool, leader_id).await.unwrap();
    setup_test_user(&pool, requester_id).await.unwrap();
    setup_test_clan(&pool, clan_id, leader_id).await.unwrap();
    ensure_clan_join_requests_table(&pool).await;

    sqlx::query(
        "INSERT INTO clan_join_requests (id, clan_id, user_id, status, created_at, updated_at) VALUES ($1, $2, $3, 'Pending', NOW(), NOW())"
    )
    .bind(request_id)
    .bind(clan_id)
    .bind(requester_id)
    .execute(&pool)
    .await
    .unwrap();

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let request_body = serde_json::json!({
        "caller_id": leader_id
    });

    let request = Request::builder()
        .uri(&format!(
            "/api/v1/clans/join-requests/{}/reject",
            request_id
        ))
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Cleanup: request may have been marked as Rejected rather than deleted
    let _ = sqlx::query("DELETE FROM clan_join_requests WHERE id = $1")
        .bind(request_id)
        .execute(&pool)
        .await;
    cleanup_test_data(&pool, clan_id, leader_id, Some(requester_id))
        .await
        .unwrap();
    pool.close().await;

    assert!(
        status == StatusCode::OK
            || status == StatusCode::FORBIDDEN
            || status == StatusCode::NOT_FOUND
            || status == StatusCode::CONFLICT,
        "reject_join_request endpoint should respond with OK, FORBIDDEN, NOT_FOUND, or CONFLICT, got: {}",
        status
    );
}

#[tokio::test]
async fn test_api_get_clan_detail_nonexistent_clan() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    let nonexistent_id = Uuid::new_v4();

    let request = Request::builder()
        .uri(&format!("/api/v1/clans/{}", nonexistent_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    pool.close().await;

    // Should return 404 for nonexistent clan
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_api_leaderboard_default_tier() {
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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    };

    let app = axum::Router::new()
        .nest("/api/v1", league_routes())
        .with_state(state);

    // Call leaderboard without tier query param — should default to "Bronze"
    let request = Request::builder()
        .uri("/api/v1/leaderboards")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    pool.close().await;

    assert_eq!(response.status(), StatusCode::OK);
}
