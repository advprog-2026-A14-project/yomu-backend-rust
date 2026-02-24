use crate::{
    AppState,
    shared::{domain::base_error::AppError, utils::response::ApiResponse},
};
use axum::{Json, extract::State};

pub async fn sync_user_handler(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let response = ApiResponse::success_without_data("Test endpoint");
    Ok(Json(response))
}
