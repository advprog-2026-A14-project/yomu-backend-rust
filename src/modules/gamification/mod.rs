// Gamification Module - Achievements & Missions (Hexagonal Architecture)
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool};

// struktur data untuk json dan db
#[derive(Deserialize)]
struct UpdateAchievementReq {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

#[derive(Serialize)]
struct UserAchievementRes {
    id: i32,
    username: Option<String>,
    email: Option<String>,
    password: Option<String>, // ini buat tes postman aja berhasil berubah atau engga habis di-update
    achievements: Vec<String>,
}

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/api/achievement/:id", get(get_achievement))
        .route("/api/achievement/:id/update", post(update_achievement))
}

async fn get_achievement(
    State(pool): State<PgPool>,
    Path(user_id): Path<i32>,
) -> Result<Json<UserAchievementRes>, StatusCode> {
    // ambil data user
    // password buat tes di postman aja
    let user = sqlx::query!("SELECT id, username, email, password FROM users_achievements_dummy WHERE id = $1", user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = match user {
        Some(u) => u,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // ambil daftar achievement user tersebut
    let records = sqlx::query!("SELECT achievement_type FROM achievements_dummy WHERE user_id = $1", user_id)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let achievements = records.into_iter().filter_map(|r| r.achievement_type).collect();

    Ok(Json(UserAchievementRes {
        id: user.id,
        username: user.username,
        email: user.email,
        password: user.password, // password buat tes di postman aja
        achievements,
    }))
}

async fn update_achievement(
    State(pool): State<PgPool>,
    Path(user_id): Path<i32>,
    Json(payload): Json<UpdateAchievementReq>,
) -> StatusCode {
    if let Some(username) = &payload.username {
        let _ = sqlx::query!("UPDATE users_achievements_dummy SET username = $1 WHERE id = $2", username, user_id).execute(&pool).await;
    }
    if let Some(email) = &payload.email {
        let _ = sqlx::query!("UPDATE users_achievements_dummy SET email = $1 WHERE id = $2", email, user_id).execute(&pool).await;
    }
    if let Some(password) = &payload.password {
        let _ = sqlx::query!("UPDATE users_achievements_dummy SET password = $1 WHERE id = $2", password, user_id).execute(&pool).await;
    }

    let insert_achievement = |ach_type: &str| {
        let pool = pool.clone();
        let ach_type = ach_type.to_string();
        async move {
            match sqlx::query!(
                "INSERT INTO achievements_dummy (user_id, achievement_type) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                user_id, ach_type
            ).execute(&pool).await {
                Ok(_) => println!("Sukses menambah achievement: {}", ach_type),
                Err(e) => println!("ERROR Database saat tambah achievement: {}", e),
            }
        }
    };

    // ambil data user terbaru untuk mengecek kelengkapannya
    let current_user = sqlx::query!("SELECT username, email, password FROM users_achievements_dummy WHERE id = $1", user_id)
        .fetch_one(&pool)
        .await;

    if let Ok(u) = current_user {
        let mut completed_count = 0;

        if u.username.is_some() {
            insert_achievement("USERNAME_FILLED").await;
            completed_count += 1;
        }
        if u.email.is_some() {
            insert_achievement("EMAIL_FILLED").await;
            completed_count += 1;
        }
        if u.password.is_some() {
            insert_achievement("PASSWORD_FILLED").await;
            completed_count += 1;
        }
        if completed_count == 3 {
            insert_achievement("ALL_COMPLETED").await;
        }
    }

    StatusCode::OK
}