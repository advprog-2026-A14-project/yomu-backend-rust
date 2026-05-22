use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ToggleProfileVisibilityResponseDto {
    pub achievement_id: Uuid,
    pub is_shown_on_profile: bool,
}
