use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MemberRole {
    #[default]
    Member,
    Leader,
}

impl std::fmt::Display for MemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberRole::Leader => write!(f, "Leader"),
            MemberRole::Member => write!(f, "Member"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClanMember {
    clan_id: Uuid,
    user_id: Uuid,
    role: MemberRole,
    joined_at: DateTime<Utc>,
}

impl ClanMember {
    pub fn new(clan_id: Uuid, user_id: Uuid, role: MemberRole) -> Self {
        Self {
            clan_id,
            user_id,
            role,
            joined_at: Utc::now(),
        }
    }

    #[allow(dead_code)]
    pub fn with_joined_at(
        clan_id: Uuid,
        user_id: Uuid,
        role: MemberRole,
        joined_at: DateTime<Utc>,
    ) -> Self {
        Self {
            clan_id,
            user_id,
            role,
            joined_at,
        }
    }

    pub fn clan_id(&self) -> Uuid {
        self.clan_id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn role(&self) -> &MemberRole {
        &self.role
    }

    pub fn joined_at(&self) -> DateTime<Utc> {
        self.joined_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clan_member_fields() {
        let clan_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let member = ClanMember::new(clan_id, user_id, MemberRole::Member);

        assert_eq!(member.clan_id(), clan_id);
        assert_eq!(member.user_id(), user_id);
        assert_eq!(member.role(), &MemberRole::Member);
    }

    #[test]
    fn test_clan_member_joined_at_is_set() {
        let before = Utc::now();
        let member = ClanMember::new(Uuid::new_v4(), Uuid::new_v4(), MemberRole::Member);
        let after = Utc::now();

        assert!(member.joined_at() >= before);
        assert!(member.joined_at() <= after);
    }
}
