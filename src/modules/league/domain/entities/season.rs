use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::clan::ClanTier;

/// Represents the lifecycle status of a season.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum SeasonStatus {
    /// Season has not started yet.
    Upcoming,
    /// Season is currently in progress.
    Active,
    /// Season has ended.
    Ended,
}

/// A competitive season within a specific tier.
///
/// Seasons have a defined start and end time, and are scoped to a single
/// clan tier (Bronze, Silver, Gold, Diamond).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Season {
    id: Uuid,
    name: String,
    tier: ClanTier,
    starts_at: chrono::DateTime<chrono::Utc>,
    ends_at: chrono::DateTime<chrono::Utc>,
    is_active: bool,
}

impl Season {
    /// Creates a new season.
    ///
    /// # Panics
    ///
    /// Panics in debug if `ends_at` is not after `starts_at`.
    pub fn new(
        name: String,
        tier: ClanTier,
        starts_at: chrono::DateTime<chrono::Utc>,
        ends_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        debug_assert!(
            ends_at > starts_at,
            "Season end time must be after start time"
        );
        Self {
            id: Uuid::new_v4(),
            name,
            tier,
            starts_at,
            ends_at,
            is_active: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_id(
        id: Uuid,
        name: String,
        tier: ClanTier,
        starts_at: chrono::DateTime<chrono::Utc>,
        ends_at: chrono::DateTime<chrono::Utc>,
        is_active: bool,
    ) -> Self {
        Self {
            id,
            name,
            tier,
            starts_at,
            ends_at,
            is_active,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tier(&self) -> &ClanTier {
        &self.tier
    }

    pub fn starts_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.starts_at
    }

    pub fn ends_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.ends_at
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Determines the season status based on current time and fields.
    pub fn status(&self) -> SeasonStatus {
        let now = chrono::Utc::now();
        if !self.is_active && now < self.starts_at {
            SeasonStatus::Upcoming
        } else if self.is_active && now <= self.ends_at {
            SeasonStatus::Active
        } else {
            SeasonStatus::Ended
        }
    }
}

/// Defines the score range boundaries for a tier within a season.
///
/// Used to determine which clans qualify for promotion or demotion
/// based on their accumulated score.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct TierBoundary {
    pub min_score: i64,
    pub max_score: i64,
}

impl TierBoundary {
    /// Creates a new TierBoundary with the given score range.
    ///
    /// # Panics
    ///
    /// Panics in debug if `min_score` > `max_score`.
    pub fn new(min_score: i64, max_score: i64) -> Self {
        debug_assert!(min_score <= max_score, "min_score must be <= max_score");
        Self {
            min_score,
            max_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeDelta, Utc};

    #[test]
    fn season_new_generates_unique_id() {
        let now = Utc::now();
        let s1 = Season::new(
            "Spring 2026".to_string(),
            ClanTier::Bronze,
            now,
            now + TimeDelta::days(7),
        );
        let s2 = Season::new(
            "Spring 2026".to_string(),
            ClanTier::Bronze,
            now,
            now + TimeDelta::days(7),
        );
        assert_ne!(s1.id(), s2.id());
    }

    #[test]
    fn season_defaults_to_not_active() {
        let now = Utc::now();
        let season = Season::new(
            "Test".to_string(),
            ClanTier::Gold,
            now,
            now + TimeDelta::days(1),
        );
        assert!(!season.is_active());
    }

    #[test]
    fn season_status_upcoming_when_not_active_before_start() {
        let future = Utc::now() + TimeDelta::days(1);
        let season = Season::with_id(
            Uuid::new_v4(),
            "Future".to_string(),
            ClanTier::Silver,
            future,
            future + TimeDelta::days(7),
            false,
        );
        assert_eq!(season.status(), SeasonStatus::Upcoming);
    }

    #[test]
    fn season_status_active_when_active_and_in_range() {
        let past = Utc::now() - TimeDelta::days(1);
        let future = Utc::now() + TimeDelta::days(7);
        let season = Season::with_id(
            Uuid::new_v4(),
            "Ongoing".to_string(),
            ClanTier::Bronze,
            past,
            future,
            true,
        );
        assert_eq!(season.status(), SeasonStatus::Active);
    }

    #[test]
    fn season_status_ended_when_past_end() {
        let past_start = Utc::now() - TimeDelta::days(10);
        let past_end = Utc::now() - TimeDelta::days(1);
        let season = Season::with_id(
            Uuid::new_v4(),
            "Expired".to_string(),
            ClanTier::Gold,
            past_start,
            past_end,
            false,
        );
        assert_eq!(season.status(), SeasonStatus::Ended);
    }

    #[test]
    fn tier_boundary_valid_range() {
        let boundary = TierBoundary::new(0, 1000);
        assert_eq!(boundary.min_score, 0);
        assert_eq!(boundary.max_score, 1000);
    }

    #[test]
    fn tier_boundary_equal_bounds_allowed() {
        let boundary = TierBoundary::new(500, 500);
        assert_eq!(boundary.min_score, boundary.max_score);
    }
}
