// Gamification Use Cases - ClaimMission, CheckAchievement
pub mod claim_mission_reward;
pub mod get_daily_missions;
pub mod get_user_achievements;
pub mod sync_quiz_gamification;
pub mod toggle_achievement_profile_visibility;
pub mod create_daily_mission;
pub mod update_daily_mission;
pub mod delete_daily_mission;

pub use claim_mission_reward::ClaimMissionRewardUseCase;
pub use get_daily_missions::GetDailyMissionsUseCase;
pub use get_user_achievements::GetUserAchievementsUseCase;
pub use sync_quiz_gamification::SyncQuizGamificationUseCase;
pub use toggle_achievement_profile_visibility::ToggleAchievementProfileVisibilityUseCase;
pub use create_daily_mission::CreateDailyMissionUseCase;
pub use update_daily_mission::UpdateDailyMissionUseCase;
pub use delete_daily_mission::DeleteDailyMissionUseCase;
