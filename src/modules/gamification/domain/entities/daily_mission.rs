use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyMission {
    pub id: Uuid,
    pub description: String,
    pub target_count: i32,
    pub date: NaiveDate,
    pub reward_points: i32,
}

impl DailyMission {
    pub fn new(
        id: Uuid, 
        description: String, 
        target_count: i32, 
        date: NaiveDate, 
        reward_points: i32
    ) -> Result<Self, &'static str> {
        let mut mission = Self {
            id,
            description: String::new(),
            target_count: 1,
            date,
            reward_points: 0,
        };
        
        mission.update_details(description, target_count, date, reward_points)?;
        Ok(mission)
    }

    pub fn update_details(
        &mut self, 
        new_description: String, 
        new_target: i32, 
        new_date: NaiveDate,
        new_reward: i32
    ) -> Result<(), &'static str> {
        
        if new_description.trim().is_empty() {
            return Err("Deskripsi misi harian tidak boleh kosong.");
        }
        if new_target <= 0 {
            return Err("Target misi harian harus lebih dari 0.");
        }
        if new_reward < 0 {
            return Err("Poin reward tidak boleh bernilai negatif.");
        }

        self.description = new_description;
        self.target_count = new_target;
        self.date = new_date;
        self.reward_points = new_reward;

        Ok(())
    }

    pub fn id(&self) -> Uuid { self.id }
    pub fn description(&self) -> &str { &self.description }
    pub fn target_count(&self) -> i32 { self.target_count }
    pub fn date(&self) -> chrono::NaiveDate { self.date }
    pub fn reward_points(&self) -> i32 { self.reward_points }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_mission_creation_success() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        let mission = DailyMission::new(Uuid::new_v4(), "Baca 3 Berita".to_string(), 3, date, 100);
        
        assert!(mission.is_ok());
        let mission = mission.unwrap();
        assert_eq!(mission.target_count(), 3);
        assert_eq!(mission.reward_points(), 100);
    }

    #[test]
    fn test_daily_mission_fails_on_invalid_input() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        let invalid_reward = DailyMission::new(Uuid::new_v4(), "Valid".to_string(), 1, date, -10);
        assert_eq!(invalid_reward.unwrap_err(), "Poin reward tidak boleh bernilai negatif.");
    }
}