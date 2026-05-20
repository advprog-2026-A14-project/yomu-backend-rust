use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClanBuff {
    id: Uuid,
    clan_id: Uuid,
    buff_name: String,
    multiplier: f64,
    is_active: bool,
    is_debuff: bool,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl ClanBuff {
    pub fn new(
        id: Uuid,
        clan_id: Uuid,
        buff_name: String,
        multiplier: f64,
        is_active: bool,
        is_debuff: bool,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            clan_id,
            buff_name,
            multiplier,
            is_active,
            is_debuff,
            expires_at,
            created_at,
        }
    }

    pub fn new_productivity_buff(clan_id: Uuid, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            clan_id,
            buff_name: "Productivity Buff".to_string(),
            multiplier: 1.2,
            is_active: true,
            is_debuff: false,
            expires_at,
            created_at: Utc::now(),
        }
    }

    pub fn new_low_accuracy_debuff(clan_id: Uuid, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            clan_id,
            buff_name: "Low Accuracy Penalty".to_string(),
            multiplier: 0.8,
            is_active: true,
            is_debuff: true,
            expires_at,
            created_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn clan_id(&self) -> Uuid {
        self.clan_id
    }

    pub fn buff_name(&self) -> &str {
        &self.buff_name
    }

    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_debuff(&self) -> bool {
        self.is_debuff
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn productivity_buff_has_correct_multiplier() {
        let clan_id = Uuid::new_v4();
        let expires = Utc::now() + chrono::Duration::hours(24);
        let buff = ClanBuff::new_productivity_buff(clan_id, expires);

        assert_eq!(buff.buff_name(), "Productivity Buff");
        assert!((buff.multiplier() - 1.2).abs() < f64::EPSILON);
        assert!(!buff.is_debuff());
        assert!(buff.is_active());
    }

    #[test]
    fn low_accuracy_debuff_has_correct_multiplier() {
        let clan_id = Uuid::new_v4();
        let expires = Utc::now() + chrono::Duration::hours(24);
        let buff = ClanBuff::new_low_accuracy_debuff(clan_id, expires);

        assert_eq!(buff.buff_name(), "Low Accuracy Penalty");
        assert!((buff.multiplier() - 0.8).abs() < f64::EPSILON);
        assert!(buff.is_debuff());
        assert!(buff.is_active());
    }

    #[test]
    fn is_expired_returns_true_for_past_expiry() {
        let clan_id = Uuid::new_v4();
        let expires = Utc::now() - chrono::Duration::hours(1);
        let buff = ClanBuff::new_productivity_buff(clan_id, expires);

        assert!(buff.is_expired());
    }

    #[test]
    fn is_expired_returns_false_for_future_expiry() {
        let clan_id = Uuid::new_v4();
        let expires = Utc::now() + chrono::Duration::hours(24);
        let buff = ClanBuff::new_productivity_buff(clan_id, expires);

        assert!(!buff.is_expired());
    }
}
