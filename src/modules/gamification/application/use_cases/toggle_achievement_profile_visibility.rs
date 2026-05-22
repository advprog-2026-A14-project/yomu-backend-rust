use std::sync::Arc;
use uuid::Uuid;

use crate::modules::gamification::application::dto::toggle_profile_visibility_response::ToggleProfileVisibilityResponseDto;
use crate::modules::gamification::domain::repositories::achievement_repository::AchievementRepository;

pub struct ToggleAchievementProfileVisibilityUseCase {
    pub repository: Arc<dyn AchievementRepository>,
}

impl ToggleAchievementProfileVisibilityUseCase {
    pub fn new(repository: Arc<dyn AchievementRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        achievement_id: Uuid,
        is_shown_on_profile: bool,
    ) -> Result<ToggleProfileVisibilityResponseDto, String> {
        let mut user_achievement = self
            .repository
            .get_user_achievement(user_id, achievement_id)
            .await?
            .ok_or_else(|| "Pencapaian pengguna tidak ditemukan.".to_string())?;

        user_achievement.set_shown_on_profile(is_shown_on_profile)?;

        self.repository
            .save_user_achievement(&user_achievement)
            .await?;

        Ok(ToggleProfileVisibilityResponseDto {
            achievement_id,
            is_shown_on_profile: user_achievement.is_shown_on_profile(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::gamification::domain::entities::user_achievement::UserAchievement;
    use crate::modules::gamification::domain::repositories::achievement_repository::MockAchievementRepository;
    use chrono::Utc;

    fn completed_user_achievement(user_id: Uuid, achievement_id: Uuid) -> UserAchievement {
        let mut ua = UserAchievement::new(user_id, achievement_id);
        ua.add_progress(1, 1, Utc::now());
        ua
    }

    #[tokio::test]
    async fn show_on_profile_when_completed_succeeds() {
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();
        let ua = completed_user_achievement(user_id, achievement_id);

        let mut mock_repo = MockAchievementRepository::new();
        mock_repo
            .expect_get_user_achievement()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(achievement_id),
            )
            .return_once(move |_, _| Ok(Some(ua)));
        mock_repo
            .expect_save_user_achievement()
            .times(1)
            .returning(|saved| {
                assert!(saved.is_shown_on_profile());
                Ok(())
            });

        let use_case = ToggleAchievementProfileVisibilityUseCase::new(Arc::new(mock_repo));
        let result = use_case
            .execute(user_id, achievement_id, true)
            .await
            .unwrap();

        assert!(result.is_shown_on_profile);
        assert_eq!(result.achievement_id, achievement_id);
    }

    #[tokio::test]
    async fn hide_on_profile_succeeds() {
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();
        let mut ua = completed_user_achievement(user_id, achievement_id);
        ua.set_shown_on_profile(true).unwrap();

        let mut mock_repo = MockAchievementRepository::new();
        mock_repo
            .expect_get_user_achievement()
            .return_once(move |_, _| Ok(Some(ua)));
        mock_repo
            .expect_save_user_achievement()
            .times(1)
            .returning(|saved| {
                assert!(!saved.is_shown_on_profile());
                Ok(())
            });

        let use_case = ToggleAchievementProfileVisibilityUseCase::new(Arc::new(mock_repo));
        let result = use_case
            .execute(user_id, achievement_id, false)
            .await
            .unwrap();

        assert!(!result.is_shown_on_profile);
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let mut mock_repo = MockAchievementRepository::new();
        mock_repo
            .expect_get_user_achievement()
            .return_once(|_, _| Ok(None));

        let use_case = ToggleAchievementProfileVisibilityUseCase::new(Arc::new(mock_repo));
        let result = use_case
            .execute(Uuid::new_v4(), Uuid::new_v4(), true)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("tidak ditemukan"));
    }

    #[tokio::test]
    async fn show_incomplete_returns_error() {
        let user_id = Uuid::new_v4();
        let achievement_id = Uuid::new_v4();
        let ua = UserAchievement::new(user_id, achievement_id);

        let mut mock_repo = MockAchievementRepository::new();
        mock_repo
            .expect_get_user_achievement()
            .return_once(move |_, _| Ok(Some(ua)));

        let use_case = ToggleAchievementProfileVisibilityUseCase::new(Arc::new(mock_repo));
        let result = use_case
            .execute(user_id, achievement_id, true)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sudah selesai"));
    }
}
