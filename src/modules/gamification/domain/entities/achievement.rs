use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: Uuid,
    pub name: String,
    pub milestone_target: i32,
    pub achievement_type: AchievementType,
    pub reward_points: i32,
}

impl Achievement {
    pub fn new(id: Uuid, name: String, target: i32, achievement_type: AchievementType, reward: i32) -> Result<Self, &'static str> {
        let mut achievement = Self {
            id,
            name: String::new(),
            milestone_target: 1,
            achievement_type,
            reward_points: 0,
        };
        
        achievement.update_details(name, target, reward)?;
        Ok(achievement)
    }

    pub fn update_details(
        &mut self, 
        new_name: String, 
        new_target: i32, 
        new_reward: i32
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

    pub fn id(&self) -> Uuid { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn milestone_target(&self) -> i32 { self.milestone_target }
    pub fn achievement_type(&self) -> &AchievementType { &self.achievement_type }
    pub fn reward_points(&self) -> i32 { self.reward_points }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_creation_success() {
        let id = Uuid::new_v4();
        let ach = Achievement::new(id, "Tes Achievement".to_string(), 10, AchievementType::Epic, 50);
        assert!(ach.is_ok());
        let ach = ach.unwrap();
        assert_eq!(ach.name(), "Tes Achievement");
        assert_eq!(ach.reward_points(), 50);
    }

    #[test]
    fn test_achievement_creation_fails_on_invalid_input() {
        let id = Uuid::new_v4();
        let empty_name = Achievement::new(id, "".to_string(), 10, AchievementType::Common, 50);
        assert_eq!(empty_name.unwrap_err(), "Nama achievement tidak boleh kosong.");

        let negative_target = Achievement::new(id, "Valid".to_string(), 0, AchievementType::Common, 50);
        assert_eq!(negative_target.unwrap_err(), "Target milestone harus lebih dari 0.");
    }
}