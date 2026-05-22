use std::sync::Arc;
use uuid::Uuid;

use crate::modules::gamification::application::dto::{
    CreateAchievementRequestDto, CreateAchievementResponseDto,
};
use crate::modules::gamification::domain::entities::achievement::{
    Achievement, AchievementTriggerType, AchievementType,
};
use crate::modules::gamification::domain::repositories::achievement_repository::AchievementRepository;

pub struct CreateAchievementUseCase {
    repository: Arc<dyn AchievementRepository>,
}

impl CreateAchievementUseCase {
    pub fn new(repository: Arc<dyn AchievementRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        dto: CreateAchievementRequestDto,
    ) -> Result<CreateAchievementResponseDto, String> {
        let achievement_type = match dto.achievement_type.as_str() {
            "Rare" | "rare" => AchievementType::Rare,
            "Epic" | "epic" => AchievementType::Epic,
            "Legendary" | "legendary" => AchievementType::Legendary,
            _ => AchievementType::Common,
        };

        let trigger_type = match dto.trigger_type.as_str() {
            "ReadArticle" | "read_article" => AchievementTriggerType::ReadArticle,
            "DailyLogin" | "daily_login" => AchievementTriggerType::DailyLogin,
            "QuizComplete" | "quiz_complete" => AchievementTriggerType::QuizComplete,
            other => {
                return Err(format!(
                    "Tipe trigger achievement tidak valid: {}. Gunakan: QuizComplete, ReadArticle, DailyLogin.",
                    other
                ))
            }
        };

        let achievement = Achievement::new(
            Uuid::new_v4(),
            dto.name,
            dto.milestone_target,
            achievement_type,
            trigger_type,
            dto.reward_points,
        )
        .map_err(|e| e.to_string())?;

        self.repository.create_achievement(&achievement).await?;

        Ok(CreateAchievementResponseDto {
            achievement_id: achievement.id(),
            name: achievement.name().to_string(),
            milestone_target: achievement.milestone_target(),
            achievement_type: achievement.achievement_type().to_string(),
            trigger_type: achievement.trigger_type().to_string(),
            reward_points: achievement.reward_points(),
        })
    }
}
