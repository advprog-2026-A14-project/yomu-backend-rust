// Unit tests for Gamification domain
mod achievement_test {
    use yomu_backend_rust::modules::gamification::domain::entities::{
        achievement::{Achievement, AchievementType},
        daily_mission::{DailyMission},
        user_achievement::{UserAchievement},
        user_mission::{UserMission},
    };
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    #[test]
    fn test_achievement() {
        let ach_id = Uuid::new_v4();
        let achievement = Achievement::new(
            ach_id,
            "Tes Achievement".to_string(),
            5,
            AchievementType::Rare,
            150,
        ).expect("Gagal membuat Achievement valid");

        assert_eq!(achievement.reward_points(), 150);
        assert_eq!(achievement.milestone_target(), 5);

        let user_id = Uuid::new_v4();
        let mut user_ach = UserAchievement::new(user_id, achievement.id());

        user_ach.add_progress(3, achievement.milestone_target());
        assert_eq!(user_ach.current_progress(), 3);
        assert!(!user_ach.is_completed());

        user_ach.add_progress(3, achievement.milestone_target());
        
        assert_eq!(user_ach.current_progress(), 5);
        assert!(user_ach.is_completed());
        assert!(user_ach.completed_at().is_some());
    }
}

mod misson_test {
    use yomu_backend_rust::modules::gamification::domain::entities::{
        achievement::{Achievement, AchievementType},
        daily_mission::{DailyMission},
        user_achievement::{UserAchievement},
        user_mission::{UserMission},
    };
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_daily_mission() {
        let mission_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        let mission = DailyMission::new(
            mission_id,
            "Baca 2 Artikel".to_string(),
            2,
            date,
            50,
        ).expect("Gagal membuat Daily Mission valid");

        assert_eq!(mission.reward_points(), 50);

        let user_id = Uuid::new_v4();
        let mut user_mission = UserMission::new(user_id, mission.id());

        user_mission.add_progress(1, mission.target_count());
        
        let failed_claim = user_mission.claim_reward(mission.target_count());
        assert!(failed_claim.is_err());
        assert_eq!(failed_claim.unwrap_err(), "Misi belum selesai, tidak bisa claim reward.");

        user_mission.add_progress(1, mission.target_count());

        let success_claim = user_mission.claim_reward(mission.target_count());
        assert!(success_claim.is_ok());
        assert!(user_mission.is_claimed());

        let double_claim = user_mission.claim_reward(mission.target_count());
        assert!(double_claim.is_err());
        assert_eq!(double_claim.unwrap_err(), "Reward untuk misi ini sudah di-claim sebelumnya.");
    }
}
