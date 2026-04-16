use crate::{
    AppState,
    shared::{domain::base_error::AppError, utils::response::ApiResponse},
};
use axum::{Json, extract::State};

#[utoipa::path(
    post,
    path = "/api/internal/users/sync",
    responses(
        (status = 200, description = "User synced successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "users"
)]
pub async fn sync_user_handler(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let response = ApiResponse::success_without_data("Test endpoint");
    Ok(Json(response))
}
