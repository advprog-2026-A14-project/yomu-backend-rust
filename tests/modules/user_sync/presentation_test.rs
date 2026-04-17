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

async fn setup_shadow_user(pool: &sqlx::PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO shadow_users (user_id, total_score, created_at) VALUES ($1, 0, NOW())",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn cleanup_test_data(pool: &sqlx::PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    let _ = sqlx::query("DELETE FROM quiz_history WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM shadow_users WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    Ok(())
}

async fn setup_app_state() -> yomu_backend_rust::AppState {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(TEST_DATABASE_URL)
        .await
        .unwrap();

    let redis = redis::Client::open("redis://localhost:6379")
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .unwrap();

    yomu_backend_rust::AppState {
        db: pool,
        redis,
        metrics: std::sync::Arc::new(yomu_backend_rust::AppMetrics::default()),
    }
}

#[tokio::test]
async fn test_api_sync_user_route_exists() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": user_id
    });

    let request = Request::builder()
        .uri("/api/internal/users/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_api_sync_user_with_valid_user_id() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": user_id
    });

    let request = Request::builder()
        .uri("/api/internal/users/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;

    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_api_sync_quiz_history_route_exists() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();
    setup_shadow_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": user_id,
        "article_id": Uuid::new_v4(),
        "score": 85,
        "accuracy": 0.92
    });

    let request = Request::builder()
        .uri("/api/internal/quiz-history/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_api_sync_quiz_history_with_all_fields() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    let article_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();
    setup_shadow_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": user_id,
        "article_id": article_id,
        "score": 100,
        "accuracy": 1.0
    });

    let request = Request::builder()
        .uri("/api/internal/quiz-history/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;

    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_api_sync_quiz_history_returns_500_for_nonexistent_user() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let non_existent_user_id = Uuid::new_v4();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": non_existent_user_id,
        "article_id": Uuid::new_v4(),
        "score": 50,
        "accuracy": 0.75
    });

    let request = Request::builder()
        .uri("/api/internal/quiz-history/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    state.db.close().await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_api_sync_quiz_history_success_with_shadow_user() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    let article_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();
    setup_shadow_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": user_id,
        "article_id": article_id,
        "score": 75,
        "accuracy": 0.88
    });

    let request = Request::builder()
        .uri("/api/internal/quiz-history/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;

    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_api_sync_user_creates_shadow_user() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": user_id
    });

    let request = Request::builder()
        .uri("/api/internal/users/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    app.oneshot(request).await.unwrap();

    let exists: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM shadow_users WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(&state.db)
            .await
            .unwrap();

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;

    assert!(exists.0, "Shadow user should be created after sync");
}

#[tokio::test]
async fn test_api_sync_user_idempotent() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();
    setup_shadow_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": user_id
    });

    let request = Request::builder()
        .uri("/api/internal/users/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response1 = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response1.status(), StatusCode::CREATED);

    let request2 = Request::builder()
        .uri("/api/internal/users/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::CREATED);

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;
}

#[tokio::test]
async fn test_api_sync_quiz_history_with_different_scores() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();
    setup_shadow_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let test_cases = vec![(100, 1.0), (0, 0.0), (50, 0.75), (1, 0.5)];

    for (score, accuracy) in test_cases {
        let request_body = serde_json::json!({
            "user_id": user_id,
            "article_id": Uuid::new_v4(),
            "score": score,
            "accuracy": accuracy
        });

        let request = Request::builder()
            .uri("/api/internal/quiz-history/sync")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();

        if status != StatusCode::CREATED {
            cleanup_test_data(&state.db, user_id).await.unwrap();
            state.db.close().await;
            panic!(
                "Expected CREATED for score={}, accuracy={}, got {:?}",
                score, accuracy, status
            );
        }
    }

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;
}

#[tokio::test]
async fn test_api_routes_return_correct_content_type() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": user_id
    });

    let request = Request::builder()
        .uri("/api/internal/users/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let content_type = response.headers().get("content-type");
    assert!(
        content_type.is_some(),
        "Response should have content-type header"
    );

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;
}

#[tokio::test]
async fn test_api_sync_quiz_history_response_body_is_valid_json() {
    use yomu_backend_rust::modules::user_sync::presentation::routes::user_sync_routes;

    let state = setup_app_state().await;

    let user_id = Uuid::new_v4();
    let article_id = Uuid::new_v4();
    setup_test_user(&state.db, user_id).await.unwrap();
    setup_shadow_user(&state.db, user_id).await.unwrap();

    let app = axum::Router::new()
        .nest("/api/internal", user_sync_routes())
        .with_state(state.clone());

    let request_body = serde_json::json!({
        "user_id": user_id,
        "article_id": article_id,
        "score": 85,
        "accuracy": 0.92
    });

    let request = Request::builder()
        .uri("/api/internal/quiz-history/sync")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    cleanup_test_data(&state.db, user_id).await.unwrap();
    state.db.close().await;

    assert_eq!(status, StatusCode::CREATED);

    let json: serde_json::Value =
        serde_json::from_str(&body_str).expect("Response should be valid JSON");
    assert!(
        json.get("success").is_some(),
        "Response should have 'success' field"
    );
}
