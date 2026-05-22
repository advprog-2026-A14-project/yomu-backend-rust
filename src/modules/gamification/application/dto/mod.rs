// Gamification DTOs - Request/Response shapes
pub mod quiz_sync;
pub mod daily_mission_response;
pub mod user_achievement_response;
pub mod toggle_profile_visibility_request;
pub mod toggle_profile_visibility_response;
pub mod create_achievement_request;
pub mod create_achievement_response;
pub mod daily_mission_admin_request;

pub use daily_mission_response::{DailyMissionItemDto, DailyMissionsResponseDto};
pub use user_achievement_response::{UserAchievementItemDto, UserAchievementsResponseDto};
pub use toggle_profile_visibility_request::ToggleProfileVisibilityRequestDto;
pub use toggle_profile_visibility_response::ToggleProfileVisibilityResponseDto;
pub use create_achievement_request::CreateAchievementRequestDto;
pub use create_achievement_response::CreateAchievementResponseDto;
pub use daily_mission_admin_request::DailyMissionAdminRequestDto;
