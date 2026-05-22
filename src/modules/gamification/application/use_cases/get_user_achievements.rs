use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::modules::gamification::application::dto::{
    UserAchievementItemDto, UserAchievementsResponseDto,
};
use crate::modules::gamification::domain::repositories::achievement_repository::AchievementRepository;

pub struct GetUserAchievementsUseCase {
    pub repository: Arc<dyn AchievementRepository>,
}

impl GetUserAchievementsUseCase {
    pub fn new(repository: Arc<dyn AchievementRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        include_hidden: bool,
    ) -> Result<UserAchievementsResponseDto, String> {
        let user_achievements = self.repository.get_user_achievements(user_id).await?;

        let visible_completed: Vec<_> = user_achievements
            .into_iter()
            .filter(|ua| ua.is_completed())
            .filter(|ua| include_hidden || ua.is_shown_on_profile())
            .collect();

        if visible_completed.is_empty() {
            return Ok(UserAchievementsResponseDto {
                user_id,
                achievements: Vec::new(),
            });
        }

        let achievement_ids: Vec<_> = visible_completed
            .iter()
            .map(|ua| ua.achievement_id())
            .collect();

        let masters = self.repository.get_achievements_by_ids(&achievement_ids).await?;
        let master_map: HashMap<_, _> = masters.into_iter().map(|a| (a.id(), a)).collect();

        let achievements = visible_completed
            .into_iter()
            .filter_map(|ua| {
                master_map.get(&ua.achievement_id()).map(|master| UserAchievementItemDto {
                    achievement_id: ua.achievement_id(),
                    name: master.name().to_string(),
                    milestone_target: master.milestone_target(),
                    current_progress: ua.current_progress(),
                    is_completed: ua.is_completed(),
                    is_shown_on_profile: ua.is_shown_on_profile(),
                    completed_at: ua.completed_at(),
                    achievement_type: master.achievement_type().to_string(),
                    reward_points: master.reward_points(),
                })
            })
            .collect();

        Ok(UserAchievementsResponseDto {
            user_id,
            achievements,
        })
    }
}