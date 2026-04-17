use std::sync::Arc;
use axum::{extract::State, Json, http::{StatusCode, HeaderMap}};
use crate::modules::gamification::presentation::routes::GamificationState;

use crate::modules::gamification::application::dto::quiz_sync::SyncQuizHistoryRequestDto;
//use crate::modules::gamification::application::use_cases::sync_quiz_gamification::SyncQuizGamificationUseCase;
use crate::shared::utils::response::ApiResponse; 

// handler POST /api/internal/quiz-history/sync
pub async fn sync_quiz_history(
    State(state): State<Arc<GamificationState>>,
    headers: HeaderMap,
    Json(payload): Json<SyncQuizHistoryRequestDto>,
) -> (StatusCode, Json<ApiResponse<()>>) {

    let api_key = headers.get("x-api-key").and_then(|v| v.to_str().ok());
    let expected_key = std::env::var("INTERNAL_API_KEY").unwrap_or_else(|_| "secret-key-default".to_string());

    if api_key != Some(&expected_key) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                success: false,
                message: "Akses ditolak: x-api-key tidak valid atau hilang".to_string(),
                data: None,
            }),
        );
    }
    
    match state.sync_quiz_use_case.execute(payload).await {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                message: "Data riwayat kuis berhasil dicatat dan diproses oleh Engine".to_string(),
                data: None, // api contract bilang tidak ada data yang dikembalikan 
            };
            (StatusCode::CREATED, Json(response)) // 201
        },
        Err(err_msg) => {
            let response = ApiResponse {
                success: false,
                message: err_msg,
                data: None,
            };
            // untuk sementara asumsikan Internal Server Error  
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}