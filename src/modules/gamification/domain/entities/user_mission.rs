use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMission {
    pub user_id: Uuid,
    pub mission_id: Uuid,
    pub current_progress: i32,
    pub is_claimed: bool,
}

impl UserMission {
    pub fn new(user_id: Uuid, mission_id: Uuid) -> Self {
        Self {
            user_id,
            mission_id,
            current_progress: 0,
            is_claimed: false,
        }
    }

    pub fn add_progress(&mut self, amount: i32, target_count: i32) {
        if amount <= 0 { return; }
        if self.current_progress < target_count {
            self.current_progress += amount;
            if self.current_progress > target_count {
                self.current_progress = target_count;
            }
        }
    }

    pub fn claim_reward(&mut self, target_count: i32) -> Result<(), &'static str> {
        if self.current_progress < target_count {
            return Err("Misi belum selesai, tidak bisa claim reward.");
        }
        if self.is_claimed {
            return Err("Reward untuk misi ini sudah di-claim sebelumnya.");
        }

        self.is_claimed = true;
        Ok(())
    }

    pub fn user_id(&self) -> Uuid { self.user_id }
    pub fn mission_id(&self) -> Uuid { self.mission_id }
    pub fn current_progress(&self) -> i32 { self.current_progress }
    pub fn is_claimed(&self) -> bool { self.is_claimed }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_claim_reward_success_when_target_met() {
        let mut user_mission = UserMission::new(Uuid::new_v4(), Uuid::new_v4());
        let target = 3;

        user_mission.add_progress(3, target);
        let result = user_mission.claim_reward(target);
        
        assert!(result.is_ok());
        assert!(user_mission.is_claimed());
    }

    #[test]
    fn test_claim_reward_fails_if_already_claimed() {
        let mut user_mission = UserMission::new(Uuid::new_v4(), Uuid::new_v4());
        let target = 3;

        user_mission.add_progress(3, target);
        let _ = user_mission.claim_reward(target); 
        let result_second_claim = user_mission.claim_reward(target); 
         
        assert!(result_second_claim.is_err());
        assert_eq!(result_second_claim.unwrap_err(), "Reward untuk misi ini sudah di-claim sebelumnya.");
    }

    #[test]
    fn test_add_progress_should_ignore_negative_or_zero_amount() {
        let mut user_mission = UserMission::new(Uuid::new_v4(), Uuid::new_v4());
        let target = 3;

        user_mission.add_progress(0, target);
        assert_eq!(user_mission.current_progress(), 0);

        user_mission.add_progress(-2, target);
        assert_eq!(user_mission.current_progress(), 0);
        
        user_mission.add_progress(1, target);
        assert_eq!(user_mission.current_progress(), 1);
    }
}