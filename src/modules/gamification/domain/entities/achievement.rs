use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AchievementType {
    #[default]
    Common,
    Rare,
    Epic,
    Legendary,
}

impl std::fmt::Display for AchievementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AchievementType::Common => write!(f, "Common"),
            AchievementType::Rare => write!(f, "Rare"),
            AchievementType::Epic => write!(f, "Epic"),
            AchievementType::Legendary => write!(f, "Legendary"),
        }
    }
}

/// Defines which user event increments this achievement's progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AchievementTriggerType {
    /// Triggered every time the user completes a quiz (most common).
    #[default]
    QuizComplete,
    /// Triggered every time the user reads an article (completing a quiz counts as reading).
    ReadArticle,
    /// Triggered on a daily login event (not yet dispatched; reserved for future use).
    DailyLogin,
}

impl std::fmt::Display for AchievementTriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AchievementTriggerType::QuizComplete => write!(f, "QuizComplete"),
            AchievementTriggerType::ReadArticle => write!(f, "ReadArticle"),
            AchievementTriggerType::DailyLogin => write!(f, "DailyLogin"),
        }
    }
}

impl AchievementTriggerType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "ReadArticle" => Self::ReadArticle,
            "DailyLogin" => Self::DailyLogin,
            _ => Self::QuizComplete,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: Uuid,
    pub name: String,
    pub milestone_target: i32,
    pub achievement_type: AchievementType,
    pub trigger_type: AchievementTriggerType,
    pub reward_points: i32,
}

impl Achievement {
    pub fn new(
        id: Uuid,
        name: String,
        target: i32,
        achievement_type: AchievementType,
        trigger_type: AchievementTriggerType,
        reward: i32,
    ) -> Result<Self, &'static str> {
        let mut achievement = Self {
            id,
            name: String::new(),
            milestone_target: 1,
            achievement_type,
            trigger_type,
            reward_points: 0,
        };

        achievement.update_details(name, target, reward)?;
        Ok(achievement)
    }

    pub fn update_details(
        &mut self,
        new_name: String,
        new_target: i32,
        new_reward: i32,
    ) -> Result<(), &'static str> {
        if new_name.trim().is_empty() {
            return Err("Nama achievement tidak boleh kosong.");
        }
        if new_target <= 0 {
            return Err("Target milestone harus lebih dari 0.");
        }
        if new_reward < 0 {
            return Err("Poin reward tidak boleh bernilai negatif.");
        }

        self.name = new_name;
        self.milestone_target = new_target;
        self.reward_points = new_reward;

        Ok(())
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn milestone_target(&self) -> i32 {
        self.milestone_target
    }
    pub fn achievement_type(&self) -> &AchievementType {
        &self.achievement_type
    }
    pub fn trigger_type(&self) -> &AchievementTriggerType {
        &self.trigger_type
    }
    pub fn reward_points(&self) -> i32 {
        self.reward_points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_creation_success() {
        let id = Uuid::new_v4();
        let ach = Achievement::new(
            id,
            "Tes Achievement".to_string(),
            10,
            AchievementType::Epic,
            AchievementTriggerType::QuizComplete,
            50,
        );
        assert!(ach.is_ok());
        let ach = ach.unwrap();
        assert_eq!(ach.name(), "Tes Achievement");
        assert_eq!(ach.reward_points(), 50);
        assert_eq!(*ach.trigger_type(), AchievementTriggerType::QuizComplete);
    }

    #[test]
    fn test_achievement_trigger_type_display() {
        assert_eq!(AchievementTriggerType::QuizComplete.to_string(), "QuizComplete");
        assert_eq!(AchievementTriggerType::ReadArticle.to_string(), "ReadArticle");
        assert_eq!(AchievementTriggerType::DailyLogin.to_string(), "DailyLogin");
    }

    #[test]
    fn test_achievement_creation_fails_on_invalid_input() {
        let id = Uuid::new_v4();
        let empty_name = Achievement::new(
            id,
            "".to_string(),
            10,
            AchievementType::Common,
            AchievementTriggerType::QuizComplete,
            50,
        );
        assert_eq!(empty_name.unwrap_err(), "Nama achievement tidak boleh kosong.");

        let negative_target = Achievement::new(
            id,
            "Valid".to_string(),
            0,
            AchievementType::Common,
            AchievementTriggerType::QuizComplete,
            50,
        );
        assert_eq!(negative_target.unwrap_err(), "Target milestone harus lebih dari 0.");
    }
}
