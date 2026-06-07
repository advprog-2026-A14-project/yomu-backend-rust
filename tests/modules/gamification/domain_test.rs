// Unit tests for Gamification domain
mod achievement_test {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;
    use yomu_backend_rust::modules::gamification::domain::entities::{
        achievement::{Achievement, AchievementTriggerType, AchievementType},
        daily_mission::DailyMission,
        user_achievement::UserAchievement,
        user_mission::UserMission,
    };

    #[test]
    fn test_achievement() {
        let ach_id = Uuid::new_v4();
        let achievement = Achievement::new(
            ach_id,
            "Tes Achievement".to_string(),
            5,
            AchievementType::Rare,
            AchievementTriggerType::QuizComplete,
            150,
        )
        .expect("Gagal membuat Achievement valid");

        assert_eq!(achievement.reward_points(), 150);
        assert_eq!(achievement.milestone_target(), 5);

        let user_id = Uuid::new_v4();
        let mut user_ach = UserAchievement::new(user_id, achievement.id());

        user_ach.add_progress(3, achievement.milestone_target(), Utc::now());
        assert_eq!(user_ach.current_progress(), 3);
        assert!(!user_ach.is_completed());

        user_ach.add_progress(3, achievement.milestone_target(), Utc::now());

        assert_eq!(user_ach.current_progress(), 5);
        assert!(user_ach.is_completed());
        assert!(user_ach.completed_at().is_some());
    }
}

mod misson_test {
    use chrono::NaiveDate;
    use uuid::Uuid;
    use yomu_backend_rust::modules::gamification::domain::entities::{
        achievement::{Achievement, AchievementType},
        daily_mission::{DailyMission, MissionType},
        user_achievement::UserAchievement,
        user_mission::UserMission,
    };

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
            MissionType::ReadArticle,
        )
        .expect("Gagal membuat Daily Mission valid");

        assert_eq!(mission.reward_points(), 50);

        let user_id = Uuid::new_v4();
        let mut user_mission = UserMission::new(user_id, mission.id());

        user_mission.add_progress(1, mission.target_count());

        let failed_claim = user_mission.claim_reward(mission.target_count());
        assert!(failed_claim.is_err());
        assert_eq!(
            failed_claim.unwrap_err(),
            "Misi belum selesai, tidak bisa claim reward."
        );

        user_mission.add_progress(1, mission.target_count());

        let success_claim = user_mission.claim_reward(mission.target_count());
        assert!(success_claim.is_ok());
        assert!(user_mission.is_claimed());

        let double_claim = user_mission.claim_reward(mission.target_count());
        assert!(double_claim.is_err());
        assert_eq!(
            double_claim.unwrap_err(),
            "Reward untuk misi ini sudah di-claim sebelumnya."
        );
    }
}

mod achievement_extra_test {
    use uuid::Uuid;
    use yomu_backend_rust::modules::gamification::domain::entities::achievement::{
        Achievement, AchievementTriggerType, AchievementType,
    };

    #[test]
    fn test_achievement_update_details_empty_name() {
        let mut ach = Achievement::new(
            Uuid::new_v4(),
            "Original".to_string(),
            5,
            AchievementType::Common,
            AchievementTriggerType::QuizComplete,
            100,
        )
        .unwrap();
        let result = ach.update_details("".to_string(), 5, 100);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Nama achievement tidak boleh kosong.");
    }

    #[test]
    fn test_achievement_update_details_invalid_target() {
        let mut ach = Achievement::new(
            Uuid::new_v4(),
            "Original".to_string(),
            5,
            AchievementType::Common,
            AchievementTriggerType::QuizComplete,
            100,
        )
        .unwrap();
        let result = ach.update_details("Valid".to_string(), 0, 100);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Target milestone harus lebih dari 0."
        );
    }

    #[test]
    fn test_achievement_update_details_negative_reward() {
        let mut ach = Achievement::new(
            Uuid::new_v4(),
            "Original".to_string(),
            5,
            AchievementType::Common,
            AchievementTriggerType::QuizComplete,
            100,
        )
        .unwrap();
        let result = ach.update_details("Valid".to_string(), 5, -1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Poin reward tidak boleh bernilai negatif."
        );
    }

    #[test]
    fn test_achievement_update_details_success() {
        let mut ach = Achievement::new(
            Uuid::new_v4(),
            "Original".to_string(),
            5,
            AchievementType::Common,
            AchievementTriggerType::QuizComplete,
            100,
        )
        .unwrap();
        let result = ach.update_details("Updated".to_string(), 10, 200);
        assert!(result.is_ok());
        assert_eq!(ach.name(), "Updated");
        assert_eq!(ach.milestone_target(), 10);
        assert_eq!(ach.reward_points(), 200);
    }

    #[test]
    fn test_achievement_type_display() {
        assert_eq!(AchievementType::Common.to_string(), "Common");
        assert_eq!(AchievementType::Rare.to_string(), "Rare");
        assert_eq!(AchievementType::Epic.to_string(), "Epic");
        assert_eq!(AchievementType::Legendary.to_string(), "Legendary");
    }

    #[test]
    fn test_achievement_trigger_type_display() {
        assert_eq!(
            AchievementTriggerType::QuizComplete.to_string(),
            "QuizComplete"
        );
        assert_eq!(
            AchievementTriggerType::ReadArticle.to_string(),
            "ReadArticle"
        );
        assert_eq!(
            AchievementTriggerType::DailyLogin.to_string(),
            "DailyLogin"
        );
    }

    #[test]
    fn test_achievement_trigger_type_from_str() {
        assert_eq!(
            AchievementTriggerType::from_str("ReadArticle"),
            AchievementTriggerType::ReadArticle
        );
        assert_eq!(
            AchievementTriggerType::from_str("DailyLogin"),
            AchievementTriggerType::DailyLogin
        );
        assert_eq!(
            AchievementTriggerType::from_str("QuizComplete"),
            AchievementTriggerType::QuizComplete
        );
        assert_eq!(
            AchievementTriggerType::from_str("Unknown"),
            AchievementTriggerType::QuizComplete
        );
    }
}

mod daily_mission_extra_test {
    use chrono::NaiveDate;
    use uuid::Uuid;
    use yomu_backend_rust::modules::gamification::domain::entities::daily_mission::{
        DailyMission, MissionType,
    };

    #[test]
    fn test_daily_mission_update_details_empty_description() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        let mut mission = DailyMission::new(
            Uuid::new_v4(),
            "Original".to_string(),
            2,
            date,
            50,
            MissionType::ReadArticle,
        )
        .unwrap();
        let result =
            mission.update_details("".to_string(), 2, date, 50, MissionType::ReadArticle);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Deskripsi misi harian tidak boleh kosong."
        );
    }

    #[test]
    fn test_daily_mission_update_details_invalid_target() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        let mut mission = DailyMission::new(
            Uuid::new_v4(),
            "Original".to_string(),
            2,
            date,
            50,
            MissionType::ReadArticle,
        )
        .unwrap();
        let result =
            mission.update_details("Valid".to_string(), 0, date, 50, MissionType::ReadArticle);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Target misi harian harus lebih dari 0."
        );
    }

    #[test]
    fn test_daily_mission_update_details_negative_reward() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        let mut mission = DailyMission::new(
            Uuid::new_v4(),
            "Original".to_string(),
            2,
            date,
            50,
            MissionType::ReadArticle,
        )
        .unwrap();
        let result =
            mission.update_details("Valid".to_string(), 2, date, -5, MissionType::ReadArticle);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Poin reward tidak boleh bernilai negatif."
        );
    }

    #[test]
    fn test_daily_mission_update_details_success() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 6).unwrap();
        let new_date = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        let mut mission = DailyMission::new(
            Uuid::new_v4(),
            "Original".to_string(),
            2,
            date,
            50,
            MissionType::ReadArticle,
        )
        .unwrap();
        let result =
            mission.update_details("Updated".to_string(), 5, new_date, 100, MissionType::Quiz);
        assert!(result.is_ok());
        assert_eq!(mission.description(), "Updated");
        assert_eq!(mission.target_count(), 5);
        assert_eq!(mission.date(), new_date);
        assert_eq!(mission.reward_points(), 100);
        assert_eq!(mission.mission_type(), MissionType::Quiz);
    }
}

mod user_achievement_extra_test {
    use chrono::Utc;
    use uuid::Uuid;
    use yomu_backend_rust::modules::gamification::domain::entities::user_achievement::UserAchievement;

    #[test]
    fn test_user_achievement_add_progress_when_already_completed() {
        let mut user_ach = UserAchievement::new(Uuid::new_v4(), Uuid::new_v4());
        user_ach.add_progress(5, 5, Utc::now());
        assert!(user_ach.is_completed());
        assert_eq!(user_ach.current_progress(), 5);
        user_ach.add_progress(3, 5, Utc::now());
        assert_eq!(user_ach.current_progress(), 5);
        assert!(user_ach.is_completed());
    }

    #[test]
    fn test_user_achievement_set_shown_on_profile_false_when_not_shown() {
        let mut user_ach = UserAchievement::new(Uuid::new_v4(), Uuid::new_v4());
        assert!(!user_ach.is_shown_on_profile());
        let result = user_ach.set_shown_on_profile(false);
        assert!(result.is_ok());
        assert!(!user_ach.is_shown_on_profile());
    }
}

mod user_mission_extra_test {
    use uuid::Uuid;
    use yomu_backend_rust::modules::gamification::domain::entities::user_mission::UserMission;

    #[test]
    fn test_user_mission_add_progress_exceeds_target_caps_at_target() {
        let mut user_mission = UserMission::new(Uuid::new_v4(), Uuid::new_v4());
        let target = 3;
        user_mission.add_progress(5, target);
        assert_eq!(user_mission.current_progress(), target);
    }
}
