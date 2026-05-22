use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ToggleProfileVisibilityRequestDto {
    pub is_shown_on_profile: bool,
}
