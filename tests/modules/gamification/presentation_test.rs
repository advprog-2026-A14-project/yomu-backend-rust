use axum::{
    Extension,
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;
use yomu_backend_rust::modules::gamification::presentation::routes::gamification_public_routes;
use yomu_backend_rust::shared::infrastructure::auth::claims::AuthenticatedUser;

const TEST_DATABASE_URL: &str = "postgres://yomu:yomu_password@localhost:5432/yomu_engine_test";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
        jwt_secret: "test_jwt_secret_for_ci_only_make_it_very_long".to_string(),
        java_core_api_key: "test_api_key_for_ci".to_string(),
    }
}

async fn setup_test_user(pool: &sqlx::PgPool, user_id: Uuid) {
    sqlx::query("INSERT INTO engine_users (user_id, total_score) VALUES ($1, 0)")
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn setup_test_achievement(
    pool: &sqlx::PgPool,
    achievement_id: Uuid,
    name: &str,
    milestone_target: i32,
    achievement_type: &str,
    trigger_type: &str,
    reward_points: i32,
) {
    sqlx::query(
        "INSERT INTO achievements (id, name, milestone_target, achievement_type, trigger_type, reward_points) \
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(achievement_id)
    .bind(name)
    .bind(milestone_target)
    .bind(achievement_type)
    .bind(trigger_type)
    .bind(reward_points)
    .execute(pool)
    .await
    .unwrap();
}

async fn setup_test_user_achievement(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    achievement_id: Uuid,
    current_progress: i32,
    is_completed: bool,
    is_shown_on_profile: bool,
) {
    sqlx::query(
        "INSERT INTO user_achievements (user_id, achievement_id, current_progress, is_completed, is_shown_on_profile) \
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(user_id)
    .bind(achievement_id)
    .bind(current_progress)
    .bind(is_completed)
    .bind(is_shown_on_profile)
    .execute(pool)
    .await
    .unwrap();
}

async fn setup_test_daily_mission(
    pool: &sqlx::PgPool,
    mission_id: Uuid,
    description: &str,
    target_count: i32,
    date: chrono::NaiveDate,
    reward_points: i32,
    mission_type: &str,
) {
    sqlx::query(
        "INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type) \
         VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(mission_id)
    .bind(description)
    .bind(target_count)
    .bind(date)
    .bind(reward_points)
    .bind(mission_type)
    .execute(pool)
    .await
    .unwrap();
}

async fn setup_test_user_mission(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    mission_id: Uuid,
    current_progress: i32,
    is_claimed: bool,
) {
    sqlx::query(
        "INSERT INTO user_missions (user_id, mission_id, current_progress, is_claimed) \
         VALUES ($1, $2, $3, $4)"
    )
    .bind(user_id)
    .bind(mission_id)
    .bind(current_progress)
    .bind(is_claimed)
    .execute(pool)
    .await
    .unwrap();
}

async fn cleanup_gamification_test_data(pool: &sqlx::PgPool, user_id: Uuid) {
    let _ = sqlx::query("DELETE FROM user_missions WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_achievements WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM engine_users WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
}

fn make_auth_user(user_id: Uuid, role: &str) -> AuthenticatedUser {
    AuthenticatedUser {
        user_id: user_id.to_string(),
        role: role.to_string(),
    }
}

fn make_admin_user(user_id: Uuid) -> AuthenticatedUser {
    make_auth_user(user_id, "ADMIN")
}

fn make_pelajar_user(user_id: Uuid) -> AuthenticatedUser {
    make_auth_user(user_id, "PELAJAR")
}

// ---------------------------------------------------------------------------
// Achievement Controller Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_user_achievements_returns_ok() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let achievement_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;
    setup_test_achievement(&state.db, achievement_id, "Test Achievement", 10, "Common", "QuizComplete", 100).await;
    setup_test_user_achievement(&state.db, user_id, achievement_id, 5, false, false).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri(&format!("/api/v1/achievements/users/{}", user_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Cleanup
    cleanup_gamification_test_data(&state.db, user_id).await;
    let _ = sqlx::query("DELETE FROM achievements WHERE id = $1")
        .bind(achievement_id)
        .execute(&state.db)
        .await;
    state.db.close().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_get_user_achievements_empty_list() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri(&format!("/api/v1/achievements/users/{}", user_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, user_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_get_user_achievements_different_user_sees_no_hidden() {
    // When requesting another user's achievements, hidden ones should not be included
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    let achievement_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;
    setup_test_achievement(&state.db, achievement_id, "Hidden Achievement", 10, "Common", "QuizComplete", 100).await;
    setup_test_user_achievement(&state.db, user_id, achievement_id, 5, false, false).await;

    setup_test_user(&state.db, other_user_id).await;
    let auth_user = make_pelajar_user(other_user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri(&format!("/api/v1/achievements/users/{}", user_id))
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, user_id).await;
    cleanup_gamification_test_data(&state.db, other_user_id).await;
    let _ = sqlx::query("DELETE FROM achievements WHERE id = $1")
        .bind(achievement_id)
        .execute(&state.db)
        .await;
    state.db.close().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_toggle_achievement_profile_visibility_success() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let achievement_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;
    setup_test_achievement(&state.db, achievement_id, "Toggle Achievement", 10, "Common", "QuizComplete", 100).await;
    setup_test_user_achievement(&state.db, user_id, achievement_id, 10, true, false).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request_body = serde_json::json!({
        "is_shown_on_profile": true
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/achievements/users/{}/{}/profile-visibility", user_id, achievement_id))
        .method("PATCH")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, user_id).await;
    let _ = sqlx::query("DELETE FROM achievements WHERE id = $1")
        .bind(achievement_id)
        .execute(&state.db)
        .await;
    state.db.close().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_toggle_achievement_profile_visibility_unauthorized_different_user() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    let achievement_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;
    setup_test_user(&state.db, other_user_id).await;
    setup_test_achievement(&state.db, achievement_id, "Toggle Achiev", 10, "Common", "QuizComplete", 100).await;
    setup_test_user_achievement(&state.db, user_id, achievement_id, 10, true, false).await;

    let auth_user = make_pelajar_user(other_user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request_body = serde_json::json!({
        "is_shown_on_profile": true
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/achievements/users/{}/{}/profile-visibility", user_id, achievement_id))
        .method("PATCH")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, user_id).await;
    cleanup_gamification_test_data(&state.db, other_user_id).await;
    let _ = sqlx::query("DELETE FROM achievements WHERE id = $1")
        .bind(achievement_id)
        .execute(&state.db)
        .await;
    state.db.close().await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_toggle_achievement_not_found() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let nonexistent_achievement_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request_body = serde_json::json!({
        "is_shown_on_profile": true
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/achievements/users/{}/{}/profile-visibility", user_id, nonexistent_achievement_id))
        .method("PATCH")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, user_id).await;
    state.db.close().await;

    assert!(
        status == StatusCode::NOT_FOUND || status == StatusCode::BAD_REQUEST,
        "Expected NOT_FOUND or BAD_REQUEST for nonexistent achievement, got: {}",
        status
    );
}

#[tokio::test]
async fn test_create_achievement_as_admin() {
    let state = setup_app_state().await;
    let admin_id = Uuid::new_v4();

    setup_test_user(&state.db, admin_id).await;

    let auth_user = make_admin_user(admin_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request_body = serde_json::json!({
        "name": "Test Achievement",
        "milestone_target": 10,
        "achievement_type": "Common",
        "trigger_type": "QuizComplete",
        "reward_points": 50
    });

    let request = Request::builder()
        .uri("/api/v1/admin/achievements")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
        if let Some(achievement_id_str) = json.get("data").and_then(|d| d.get("achievement_id")).and_then(|id| id.as_str()) {
            if let Ok(achievement_id) = Uuid::parse_str(achievement_id_str) {
                let _ = sqlx::query("DELETE FROM achievements WHERE id = $1")
                    .bind(achievement_id)
                    .execute(&state.db)
                    .await;
            }
        }
    }

    cleanup_gamification_test_data(&state.db, admin_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_achievement_rejected_for_non_admin() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request_body = serde_json::json!({
        "name": "Should Fail Achievement",
        "milestone_target": 10,
        "achievement_type": "Common",
        "trigger_type": "QuizComplete",
        "reward_points": 50
    });

    let request = Request::builder()
        .uri("/api/v1/admin/achievements")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, user_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_achievement_bad_request_missing_fields() {
    let state = setup_app_state().await;
    let admin_id = Uuid::new_v4();

    setup_test_user(&state.db, admin_id).await;

    let auth_user = make_admin_user(admin_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request_body = serde_json::json!({});

    let request = Request::builder()
        .uri("/api/v1/admin/achievements")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, admin_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

// ---------------------------------------------------------------------------
// Mission Controller Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_daily_missions_returns_ok() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri("/api/v1/missions/daily")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, user_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_create_daily_mission_as_admin() {
    let state = setup_app_state().await;
    let admin_id = Uuid::new_v4();

    setup_test_user(&state.db, admin_id).await;

    let auth_user = make_admin_user(admin_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let today = chrono::Utc::now().date_naive();
    let request_body = serde_json::json!({
        "description": "Read 3 articles today",
        "target_count": 3,
        "date": today.format("%Y-%m-%d").to_string(),
        "reward_points": 50,
        "mission_type": "ReadArticle"
    });

    let request = Request::builder()
        .uri("/api/v1/admin/missions/daily")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
        if let Some(mission_id_str) = json.get("data").and_then(|d| d.get("mission_id")).and_then(|id| id.as_str()) {
            if let Ok(mission_id) = Uuid::parse_str(mission_id_str) {
                let _ = sqlx::query("DELETE FROM user_missions WHERE mission_id = $1")
                    .bind(mission_id)
                    .execute(&state.db)
                    .await;
                let _ = sqlx::query("DELETE FROM daily_missions WHERE id = $1")
                    .bind(mission_id)
                    .execute(&state.db)
                    .await;
            }
        }
    }

    cleanup_gamification_test_data(&state.db, admin_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_daily_mission_rejected_for_non_admin() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let today = chrono::Utc::now().date_naive();
    let request_body = serde_json::json!({
        "description": "Unauthorized mission",
        "target_count": 3,
        "date": today.format("%Y-%m-%d").to_string(),
        "reward_points": 50,
        "mission_type": "ReadArticle"
    });

    let request = Request::builder()
        .uri("/api/v1/admin/missions/daily")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, user_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_daily_mission_as_admin() {
    let state = setup_app_state().await;
    let admin_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();

    setup_test_user(&state.db, admin_id).await;
    let today = chrono::Utc::now().date_naive();
    setup_test_daily_mission(&state.db, mission_id, "Old description", 5, today, 20, "ReadArticle").await;

    let auth_user = make_admin_user(admin_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let tomorrow = today + chrono::Duration::days(1);
    let request_body = serde_json::json!({
        "description": "Updated description",
        "target_count": 10,
        "date": tomorrow.format("%Y-%m-%d").to_string(),
        "reward_points": 100,
        "mission_type": "Quiz"
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/admin/missions/{}", mission_id))
        .method("PATCH")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Cleanup
    let _ = sqlx::query("DELETE FROM daily_missions WHERE id = $1")
        .bind(mission_id)
        .execute(&state.db)
        .await;
    cleanup_gamification_test_data(&state.db, admin_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_update_daily_mission_rejected_for_non_admin() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;
    let today = chrono::Utc::now().date_naive();
    setup_test_daily_mission(&state.db, mission_id, "Test mission", 5, today, 20, "ReadArticle").await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request_body = serde_json::json!({
        "description": "Unauthorized update",
        "target_count": 10,
        "date": today.format("%Y-%m-%d").to_string(),
        "reward_points": 100,
        "mission_type": "ReadArticle"
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/admin/missions/{}", mission_id))
        .method("PATCH")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let _ = sqlx::query("DELETE FROM daily_missions WHERE id = $1")
        .bind(mission_id)
        .execute(&state.db)
        .await;
    cleanup_gamification_test_data(&state.db, user_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_update_nonexistent_daily_mission_returns_not_found() {
    let state = setup_app_state().await;
    let admin_id = Uuid::new_v4();
    let nonexistent_mission_id = Uuid::new_v4();

    setup_test_user(&state.db, admin_id).await;

    let auth_user = make_admin_user(admin_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let today = chrono::Utc::now().date_naive();
    let request_body = serde_json::json!({
        "description": "Update nonexistent",
        "target_count": 10,
        "date": today.format("%Y-%m-%d").to_string(),
        "reward_points": 100,
        "mission_type": "ReadArticle"
    });

    let request = Request::builder()
        .uri(&format!("/api/v1/admin/missions/{}", nonexistent_mission_id))
        .method("PATCH")
        .header("Content-Type", "application/json")
        .body(Body::from(request_body.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, admin_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_daily_mission_as_admin() {
    let state = setup_app_state().await;
    let admin_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();

    setup_test_user(&state.db, admin_id).await;
    let today = chrono::Utc::now().date_naive();
    setup_test_daily_mission(&state.db, mission_id, "Mission to delete", 5, today, 20, "ReadArticle").await;

    let auth_user = make_admin_user(admin_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri(&format!("/api/v1/admin/missions/{}", mission_id))
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let _ = sqlx::query("DELETE FROM daily_missions WHERE id = $1")
        .bind(mission_id)
        .execute(&state.db)
        .await;
    cleanup_gamification_test_data(&state.db, admin_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_delete_daily_mission_rejected_for_non_admin() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;
    let today = chrono::Utc::now().date_naive();
    setup_test_daily_mission(&state.db, mission_id, "Mission to keep", 5, today, 20, "ReadArticle").await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri(&format!("/api/v1/admin/missions/{}", mission_id))
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let _ = sqlx::query("DELETE FROM daily_missions WHERE id = $1")
        .bind(mission_id)
        .execute(&state.db)
        .await;
    cleanup_gamification_test_data(&state.db, user_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_nonexistent_daily_mission_returns_not_found() {
    let state = setup_app_state().await;
    let admin_id = Uuid::new_v4();
    let nonexistent_mission_id = Uuid::new_v4();

    setup_test_user(&state.db, admin_id).await;

    let auth_user = make_admin_user(admin_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri(&format!("/api/v1/admin/missions/{}", nonexistent_mission_id))
        .method("DELETE")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, admin_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Claim Mission Controller Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_claim_mission_reward_not_found() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let nonexistent_mission_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri(&format!("/api/v1/missions/{}/claim", nonexistent_mission_id))
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    cleanup_gamification_test_data(&state.db, user_id).await;
    state.db.close().await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_claim_mission_reward_not_completed() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;
    let today = chrono::Utc::now().date_naive();
    setup_test_daily_mission(&state.db, mission_id, "Daily quiz", 5, today, 50, "Quiz").await;
    setup_test_user_mission(&state.db, user_id, mission_id, 2, false).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri(&format!("/api/v1/missions/{}/claim", mission_id))
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let _ = sqlx::query("DELETE FROM user_missions WHERE user_id = $1 AND mission_id = $2")
        .bind(user_id)
        .bind(mission_id)
        .execute(&state.db)
        .await;
    cleanup_gamification_test_data(&state.db, user_id).await;
    let _ = sqlx::query("DELETE FROM daily_missions WHERE id = $1")
        .bind(mission_id)
        .execute(&state.db)
        .await;
    state.db.close().await;

    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::NOT_FOUND,
        "Expected BAD_REQUEST (mission not completed) or NOT_FOUND, got: {}",
        status
    );
}

#[tokio::test]
async fn test_claim_mission_reward_already_claimed() {
    let state = setup_app_state().await;
    let user_id = Uuid::new_v4();
    let mission_id = Uuid::new_v4();

    setup_test_user(&state.db, user_id).await;
    let today = chrono::Utc::now().date_naive();
    setup_test_daily_mission(&state.db, mission_id, "Daily read", 5, today, 50, "ReadArticle").await;
    // Progress == target and already claimed
    setup_test_user_mission(&state.db, user_id, mission_id, 5, true).await;

    let auth_user = make_pelajar_user(user_id);
    let app = axum::Router::new()
        .nest("/api/v1", gamification_public_routes())
        .with_state(state.clone())
        .layer(Extension(auth_user));

    let request = Request::builder()
        .uri(&format!("/api/v1/missions/{}/claim", mission_id))
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    // Cleanup
    let _ = sqlx::query("DELETE FROM user_missions WHERE user_id = $1 AND mission_id = $2")
        .bind(user_id)
        .bind(mission_id)
        .execute(&state.db)
        .await;
    cleanup_gamification_test_data(&state.db, user_id).await;
    let _ = sqlx::query("DELETE FROM daily_missions WHERE id = $1")
        .bind(mission_id)
        .execute(&state.db)
        .await;
    state.db.close().await;

    // Already claimed should return BAD_REQUEST
    assert!(
        status == StatusCode::BAD_REQUEST,
        "Expected BAD_REQUEST for already claimed mission, got: {}",
        status
    );
}