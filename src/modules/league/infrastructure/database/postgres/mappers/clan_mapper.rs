use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::modules::league::domain::entities::clan::{Clan, ClanTier};
use crate::modules::league::domain::entities::clan_member::{ClanMember, MemberRole};

pub struct ClanMapper;

#[derive(FromRow)]
pub struct ClanRow {
    pub id: Uuid,
    pub name: String,
    pub leader_id: Uuid,
    pub tier: String,
    pub total_score: i64,
    pub created_at: DateTime<Utc>,
}

impl ClanMapper {
    pub fn from_row(row: &ClanRow) -> Result<Clan, String> {
        let tier = TierMapper::from_db_str(&row.tier)?;
        Ok(Clan::with_id(
            row.id,
            row.name.clone(),
            row.leader_id,
            tier,
            row.total_score,
            row.created_at,
        ))
    }

    pub fn from_rows(rows: &[ClanRow]) -> Vec<Clan> {
        rows.iter().filter_map(|r| Self::from_row(r).ok()).collect()
    }
}

pub struct ClanMemberMapper;

#[derive(FromRow)]
pub struct ClanMemberRow {
    clan_id: Uuid,
    user_id: Uuid,
    role: String,
    joined_at: DateTime<Utc>,
}

impl ClanMemberMapper {
    pub fn from_row(row: &ClanMemberRow) -> ClanMember {
        ClanMember::with_joined_at(
            row.clan_id,
            row.user_id,
            MemberRoleMapper::from_db_str(&row.role),
            row.joined_at,
        )
    }

    pub fn from_rows(rows: &[ClanMemberRow]) -> Vec<ClanMember> {
        rows.iter().map(Self::from_row).collect()
    }
}

pub struct TierMapper;

impl TierMapper {
    pub fn from_db_str(s: &str) -> Result<ClanTier, String> {
        match s.to_uppercase().as_str() {
            "BRONZE" => Ok(ClanTier::Bronze),
            "SILVER" => Ok(ClanTier::Silver),
            "GOLD" => Ok(ClanTier::Gold),
            "DIAMOND" => Ok(ClanTier::Diamond),
            _ => Err(format!("Invalid tier: {}", s)),
        }
    }

    pub fn to_db_str(tier: &ClanTier) -> &'static str {
        match tier {
            ClanTier::Bronze => "BRONZE",
            ClanTier::Silver => "SILVER",
            ClanTier::Gold => "GOLD",
            ClanTier::Diamond => "DIAMOND",
        }
    }
}

pub struct MemberRoleMapper;

impl MemberRoleMapper {
    pub fn from_db_str(s: &str) -> MemberRole {
        match s.to_uppercase().as_str() {
            "LEADER" => MemberRole::Leader,
            _ => MemberRole::Member,
        }
    }

    pub fn to_db_str(role: &MemberRole) -> &'static str {
        match role {
            MemberRole::Leader => "LEADER",
            MemberRole::Member => "MEMBER",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_mapper_from_db_str_valid() {
        assert_eq!(TierMapper::from_db_str("BRONZE").unwrap(), ClanTier::Bronze);
        assert_eq!(TierMapper::from_db_str("Silver").unwrap(), ClanTier::Silver);
        assert_eq!(TierMapper::from_db_str("GOLD").unwrap(), ClanTier::Gold);
        assert_eq!(
            TierMapper::from_db_str("Diamond").unwrap(),
            ClanTier::Diamond
        );
    }

    #[test]
    fn test_tier_mapper_from_db_str_invalid() {
        assert!(TierMapper::from_db_str("INVALID").is_err());
        assert!(TierMapper::from_db_str("").is_err());
    }

    #[test]
    fn test_tier_mapper_to_db_str() {
        assert_eq!(TierMapper::to_db_str(&ClanTier::Bronze), "BRONZE");
        assert_eq!(TierMapper::to_db_str(&ClanTier::Silver), "SILVER");
        assert_eq!(TierMapper::to_db_str(&ClanTier::Gold), "GOLD");
        assert_eq!(TierMapper::to_db_str(&ClanTier::Diamond), "DIAMOND");
    }

    #[test]
    fn test_member_role_mapper_from_db_str() {
        assert_eq!(MemberRoleMapper::from_db_str("LEADER"), MemberRole::Leader);
        assert_eq!(MemberRoleMapper::from_db_str("Member"), MemberRole::Member);
        assert_eq!(MemberRoleMapper::from_db_str("INVALID"), MemberRole::Member);
    }

    #[test]
    fn test_member_role_mapper_to_db_str() {
        assert_eq!(MemberRoleMapper::to_db_str(&MemberRole::Leader), "LEADER");
        assert_eq!(MemberRoleMapper::to_db_str(&MemberRole::Member), "MEMBER");
    }

    #[test]
    fn test_clan_mapper_from_row() {
        let row = ClanRow {
            id: Uuid::new_v4(),
            name: "Test Clan".to_string(),
            leader_id: Uuid::new_v4(),
            tier: "Gold".to_string(),
            total_score: 100,
            created_at: Utc::now(),
        };

        let clan = ClanMapper::from_row(&row).unwrap();
        assert_eq!(clan.id(), row.id);
        assert_eq!(clan.name(), "Test Clan");
        assert_eq!(clan.leader_id(), row.leader_id);
        assert_eq!(clan.tier(), &ClanTier::Gold);
        assert_eq!(clan.total_score(), 100);
    }

    #[test]
    fn test_clan_mapper_from_row_invalid_tier() {
        let row = ClanRow {
            id: Uuid::new_v4(),
            name: "Test Clan".to_string(),
            leader_id: Uuid::new_v4(),
            tier: "INVALID".to_string(),
            total_score: 100,
            created_at: Utc::now(),
        };

        assert!(ClanMapper::from_row(&row).is_err());
    }

    #[test]
    fn test_clan_member_mapper_from_row() {
        let row = ClanMemberRow {
            clan_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            role: "LEADER".to_string(),
            joined_at: Utc::now(),
        };

        let member = ClanMemberMapper::from_row(&row);
        assert_eq!(member.clan_id(), row.clan_id);
        assert_eq!(member.user_id(), row.user_id);
        assert_eq!(member.role(), &MemberRole::Leader);
    }
}
