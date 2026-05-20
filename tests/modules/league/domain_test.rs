mod clan_tests {
    use uuid::Uuid;
    use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
    use yomu_backend_rust::modules::league::domain::entities::clan::ClanTier;

    #[test]
    fn test_create_new_clan() {
        let clan_name = "Test Clan";
        let leader_id = Uuid::new_v4();

        let clan = Clan::new(clan_name.to_string(), leader_id);

        assert_eq!(
            clan.tier(),
            &ClanTier::Bronze,
            "New clan should default to Bronze tier"
        );
        assert_eq!(clan.total_score(), 0, "New clan should have 0 total score");
        assert_eq!(clan.name(), clan_name, "Clan name should match");
    }
}

mod clan_member_tests {
    use chrono::Utc;
    use uuid::Uuid;
    use yomu_backend_rust::modules::league::domain::entities::clan_member::ClanMember;
    use yomu_backend_rust::modules::league::domain::entities::clan_member::MemberRole;

    #[test]
    fn test_create_clan_member() {
        let clan_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let before_create = Utc::now();

        let member = ClanMember::new(clan_id, user_id, MemberRole::Member);

        assert_eq!(member.clan_id(), clan_id, "clan_id should match");
        assert_eq!(member.user_id(), user_id, "user_id should match");
        assert!(
            member.joined_at() >= before_create,
            "joined_at should be set"
        );
        assert_eq!(
            member.role(),
            &MemberRole::Member,
            "Default role should be Member"
        );
    }
}

mod score_calculation_tests {
    use yomu_backend_rust::modules::league::domain::entities::score::Score;

    #[test]
    fn test_score_calculation_with_buff() {
        let base_score = 100;
        let buff_multiplier = 1.5;

        let final_score = Score::calculate_with_modifier(base_score, buff_multiplier);

        assert_eq!(final_score, 150, "100 * 1.5 should equal 150");
    }

    #[test]
    fn test_score_calculation_with_debuff() {
        let base_score = 100;
        let debuff_multiplier = 0.8;

        let final_score = Score::calculate_with_modifier(base_score, debuff_multiplier);

        assert_eq!(final_score, 80, "100 * 0.8 should equal 80");
    }

    #[test]
    fn test_score_calculation_no_modifier() {
        let base_score = 100;
        let no_modifier = 1.0;

        let final_score = Score::calculate_with_modifier(base_score, no_modifier);

        assert_eq!(final_score, 100, "100 * 1.0 should equal 100");
    }
}

mod season_tests {
    use chrono::{TimeDelta, Utc};
    use uuid::Uuid;
    use yomu_backend_rust::modules::league::domain::entities::clan::ClanTier;
    use yomu_backend_rust::modules::league::domain::entities::season::{Season, SeasonStatus};

    #[test]
    fn test_season_new_creates_with_bronze_tier() {
        let now = Utc::now();
        let season = Season::new(
            "Bronze League S1".to_string(),
            ClanTier::Bronze,
            now,
            now + TimeDelta::days(30),
        );

        assert_eq!(
            season.tier(),
            &ClanTier::Bronze,
            "New season should have Bronze tier"
        );
        assert_eq!(season.name(), "Bronze League S1");
    }

    #[test]
    fn test_season_new_creates_with_silver_tier() {
        let now = Utc::now();
        let season = Season::new(
            "Silver League S1".to_string(),
            ClanTier::Silver,
            now,
            now + TimeDelta::days(30),
        );

        assert_eq!(
            season.tier(),
            &ClanTier::Silver,
            "Season should have Silver tier"
        );
    }

    #[test]
    fn test_season_new_creates_with_gold_tier() {
        let now = Utc::now();
        let season = Season::new(
            "Gold League S1".to_string(),
            ClanTier::Gold,
            now,
            now + TimeDelta::days(30),
        );

        assert_eq!(
            season.tier(),
            &ClanTier::Gold,
            "Season should have Gold tier"
        );
    }

    #[test]
    fn test_season_new_creates_with_diamond_tier() {
        let now = Utc::now();
        let season = Season::new(
            "Diamond League S1".to_string(),
            ClanTier::Diamond,
            now,
            now + TimeDelta::days(30),
        );

        assert_eq!(
            season.tier(),
            &ClanTier::Diamond,
            "Season should have Diamond tier"
        );
    }

    #[test]
    fn test_season_is_active_defaults_to_false() {
        let now = Utc::now();
        let season = Season::new(
            "Test Season".to_string(),
            ClanTier::Bronze,
            now,
            now + TimeDelta::days(7),
        );

        assert!(
            !season.is_active(),
            "New season should not be active by default"
        );
    }

    #[test]
    fn test_season_with_id_can_set_active() {
        let season = Season::with_id(
            Uuid::new_v4(),
            "Active Season".to_string(),
            ClanTier::Bronze,
            Utc::now() - TimeDelta::days(1),
            Utc::now() + TimeDelta::days(7),
            true,
        );

        assert!(
            season.is_active(),
            "Season created with is_active=true should be active"
        );
    }

    #[test]
    fn test_season_status_upcoming_before_start() {
        let future = Utc::now() + TimeDelta::days(1);
        let season = Season::with_id(
            Uuid::new_v4(),
            "Future Season".to_string(),
            ClanTier::Bronze,
            future,
            future + TimeDelta::days(7),
            false,
        );

        assert_eq!(
            season.status(),
            SeasonStatus::Upcoming,
            "Season before start with is_active=false should be Upcoming"
        );
    }

    #[test]
    fn test_season_status_active_when_in_range() {
        let past = Utc::now() - TimeDelta::days(1);
        let future = Utc::now() + TimeDelta::days(7);
        let season = Season::with_id(
            Uuid::new_v4(),
            "Ongoing Season".to_string(),
            ClanTier::Silver,
            past,
            future,
            true,
        );

        assert_eq!(
            season.status(),
            SeasonStatus::Active,
            "Active season in date range should be Active"
        );
    }

    #[test]
    fn test_season_status_ended_after_end_date() {
        let past_start = Utc::now() - TimeDelta::days(10);
        let past_end = Utc::now() - TimeDelta::days(1);
        let season = Season::with_id(
            Uuid::new_v4(),
            "Expired Season".to_string(),
            ClanTier::Gold,
            past_start,
            past_end,
            false,
        );

        assert_eq!(
            season.status(),
            SeasonStatus::Ended,
            "Past season should be Ended"
        );
    }

    #[test]
    fn test_season_id_is_unique() {
        let now = Utc::now();
        let season1 = Season::new(
            "S1".to_string(),
            ClanTier::Bronze,
            now,
            now + TimeDelta::days(7),
        );
        let season2 = Season::new(
            "S2".to_string(),
            ClanTier::Bronze,
            now,
            now + TimeDelta::days(7),
        );

        assert_ne!(
            season1.id(),
            season2.id(),
            "Each season should have a unique ID"
        );
    }

    #[test]
    fn test_season_start_and_end_times() {
        let start = Utc::now();
        let end = start + TimeDelta::days(30);
        let season = Season::new("Timed Season".to_string(), ClanTier::Bronze, start, end);

        assert_eq!(season.starts_at(), start);
        assert_eq!(season.ends_at(), end);
    }
}

mod scoring_strategy_tests {
    use uuid::Uuid;
    use yomu_backend_rust::modules::league::domain::entities::scoring_strategy::{
        ScoringStrategy, calculate_score,
    };

    fn make_scores(vals: &[i64]) -> Vec<(Uuid, i64)> {
        vals.iter().map(|&s| (Uuid::new_v4(), s)).collect()
    }

    #[test]
    fn test_bronze_sum_adds_all_member_scores() {
        let scores = make_scores(&[10, 20, 30]);
        let result = calculate_score(ScoringStrategy::BronzeSum, &scores);

        assert_eq!(result, 60, "BronzeSum should add all scores: 10+20+30=60");
    }

    #[test]
    fn test_bronze_sum_single_member() {
        let scores = make_scores(&[42]);
        let result = calculate_score(ScoringStrategy::BronzeSum, &scores);

        assert_eq!(
            result, 42,
            "BronzeSum with single member should return that score"
        );
    }

    #[test]
    fn test_bronze_sum_empty_returns_zero() {
        let scores: Vec<(Uuid, i64)> = vec![];
        let result = calculate_score(ScoringStrategy::BronzeSum, &scores);

        assert_eq!(result, 0, "BronzeSum with no members should return 0");
    }

    #[test]
    fn test_silver_weighted_avg_with_boost() {
        let scores = make_scores(&[100, 200, 300]);
        // avg = (100+200+300)/3 = 200, boosted = 200 * 1.1 = 220
        let result = calculate_score(ScoringStrategy::SilverWeightedAvg, &scores);

        assert_eq!(
            result, 220,
            "SilverWeightedAvg should return avg * 1.1 = 220"
        );
    }

    #[test]
    fn test_gold_weighted_avg_with_attendance() {
        let scores = make_scores(&[100, 200]);
        // avg = (100+200)/2 = 150, with attendance = 150 * 1.2 = 180
        let result = calculate_score(ScoringStrategy::GoldWeightedAvg, &scores);

        assert_eq!(result, 180, "GoldWeightedAvg should return avg * 1.2 = 180");
    }

    #[test]
    fn test_diamond_weighted_avg_with_recency() {
        let scores = make_scores(&[50, 50]);
        // avg = (50+50)/2 = 50, with recency = 50 * 1.5 = 75
        let result = calculate_score(ScoringStrategy::DiamondWeightedAvg, &scores);

        assert_eq!(
            result, 75,
            "DiamondWeightedAvg should return avg * 1.5 = 75"
        );
    }

    #[test]
    fn test_all_strategies_return_zero_for_empty() {
        let scores: Vec<(Uuid, i64)> = vec![];

        assert_eq!(calculate_score(ScoringStrategy::BronzeSum, &scores), 0);
        assert_eq!(
            calculate_score(ScoringStrategy::SilverWeightedAvg, &scores),
            0
        );
        assert_eq!(
            calculate_score(ScoringStrategy::GoldWeightedAvg, &scores),
            0
        );
        assert_eq!(
            calculate_score(ScoringStrategy::DiamondWeightedAvg, &scores),
            0
        );
    }

    #[test]
    fn test_scoring_strategy_display() {
        assert_eq!(ScoringStrategy::BronzeSum.to_string(), "BronzeSum");
        assert_eq!(
            ScoringStrategy::SilverWeightedAvg.to_string(),
            "SilverWeightedAvg"
        );
        assert_eq!(
            ScoringStrategy::GoldWeightedAvg.to_string(),
            "GoldWeightedAvg"
        );
        assert_eq!(
            ScoringStrategy::DiamondWeightedAvg.to_string(),
            "DiamondWeightedAvg"
        );
    }

    #[test]
    fn test_bronze_sum_large_team() {
        let scores = make_scores(&[10, 20, 30, 40, 50]);
        let result = calculate_score(ScoringStrategy::BronzeSum, &scores);

        assert_eq!(result, 150, "BronzeSum for 5 members: 10+20+30+40+50=150");
    }

    #[test]
    fn test_diamond_weighted_avg_rounding() {
        let scores = make_scores(&[100, 103]);
        // avg = (100+103)/2 = 101 (integer division), 101 * 1.5 = 151.5 -> rounds to 152
        let result = calculate_score(ScoringStrategy::DiamondWeightedAvg, &scores);

        assert_eq!(result, 152, "Diamond should round correctly");
    }
}
