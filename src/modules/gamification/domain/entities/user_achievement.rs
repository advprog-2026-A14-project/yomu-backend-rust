use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAchievement {
    pub user_id: Uuid,
    pub achievement_id: Uuid,
    pub current_progress: i32,
    pub is_completed: bool,
    pub is_shown_on_profile: bool,
    pub completed_at: Option<DateTime<Utc>>,
}

impl UserAchievement {
    pub fn new(user_id: Uuid, achievement_id: Uuid) -> Self {
        Self {
            user_id,
            achievement_id,
            current_progress: 0,
            is_completed: false,
            is_shown_on_profile: false,
            completed_at: None,
        }
    }

    pub fn add_progress(&mut self, amount: i32, milestone_target: i32, now: DateTime<Utc>) {
        if amount <= 0 {
            return;
        }
        if self.is_completed {
            return;
        }
        self.current_progress += amount;
        if self.current_progress >= milestone_target {
            self.current_progress = milestone_target;
            self.is_completed = true;
            self.completed_at = Some(now);
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn achievement_id(&self) -> Uuid {
        self.achievement_id
    }

    pub fn current_progress(&self) -> i32 {
        self.current_progress
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed
    }

    pub fn is_shown_on_profile(&self) -> bool {
        self.is_shown_on_profile
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_progress_should_complete_achievement_when_target_met() {
        let mut user_achievement = UserAchievement::new(Uuid::new_v4(), Uuid::new_v4());
        let milestone_target = 10;
        let test_time = Utc::now();

        user_achievement.add_progress(5, milestone_target, test_time);
        assert_eq!(user_achievement.current_progress(), 5);
        assert!(!user_achievement.is_completed());

        user_achievement.add_progress(7, milestone_target, test_time);
        assert_eq!(user_achievement.current_progress(), 10);
        assert!(user_achievement.is_completed());
        assert!(user_achievement.completed_at().is_some());
        assert_eq!(user_achievement.completed_at(), Some(test_time));
    }

    #[test]
    fn test_add_progress_should_ignore_negative_or_zero_amount() {
        let mut user_achievement = UserAchievement::new(Uuid::new_v4(), Uuid::new_v4());
        let milestone_target = 10;
        let test_time = Utc::now();

        user_achievement.add_progress(0, milestone_target, test_time);
        assert_eq!(user_achievement.current_progress(), 0);

        user_achievement.add_progress(-5, milestone_target, test_time);
        assert_eq!(user_achievement.current_progress(), 0);
    }
}
