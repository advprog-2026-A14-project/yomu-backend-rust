mod clan_tests {
    use yomu_backend_rust::modules::league::domain::entities::clan::Clan;
    use yomu_backend_rust::modules::league::domain::entities::clan::ClanTier;

    #[test]
    fn test_create_new_clan() {
        let clan_name = "Test Clan";

        let clan = Clan::new(clan_name.to_string());

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
