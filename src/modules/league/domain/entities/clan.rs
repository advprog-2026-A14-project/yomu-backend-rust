use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, ToSchema)]
pub enum ClanTier {
    #[default]
    Bronze,
    Silver,
    Gold,
    Diamond,
}

impl std::fmt::Display for ClanTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClanTier::Bronze => write!(f, "Bronze"),
            ClanTier::Silver => write!(f, "Silver"),
            ClanTier::Gold => write!(f, "Gold"),
            ClanTier::Diamond => write!(f, "Diamond"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Clan {
    id: Uuid,
    name: String,
    leader_id: Uuid,
    tier: ClanTier,
    total_score: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Clan {
    pub fn new(name: String, leader_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            leader_id,
            tier: ClanTier::default(),
            total_score: 0,
            created_at: chrono::Utc::now(),
        }
    }

    #[allow(dead_code)]
    pub fn with_id(
        id: Uuid,
        name: String,
        leader_id: Uuid,
        tier: ClanTier,
        total_score: i64,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id,
            name,
            leader_id,
            tier,
            total_score,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn leader_id(&self) -> Uuid {
        self.leader_id
    }

    pub fn tier(&self) -> &ClanTier {
        &self.tier
    }

    pub fn total_score(&self) -> i64 {
        self.total_score
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clan_default_tier_is_bronze() {
        let clan = Clan::new("Test Clan".to_string(), Uuid::new_v4());
        assert_eq!(clan.tier(), &ClanTier::Bronze);
    }

    #[test]
    fn test_clan_default_score_is_zero() {
        let clan = Clan::new("Test Clan".to_string(), Uuid::new_v4());
        assert_eq!(clan.total_score(), 0);
    }
}
