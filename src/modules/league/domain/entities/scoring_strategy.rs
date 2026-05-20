use uuid::Uuid;

/// Tier-weighted scoring algorithms.
///
/// Each variant implements a tier-specific formula that weights
/// member contributions differently as clans progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoringStrategy {
    /// Bronze tier: simple sum of all member scores
    BronzeSum,
    /// Silver tier: weighted average with 1.1x boost
    SilverWeightedAvg,
    /// Gold tier: weighted average with 1.2x attendance factor
    GoldWeightedAvg,
    /// Diamond tier: weighted average with 1.5x recency factor
    DiamondWeightedAvg,
}

impl std::fmt::Display for ScoringStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScoringStrategy::BronzeSum => write!(f, "BronzeSum"),
            ScoringStrategy::SilverWeightedAvg => write!(f, "SilverWeightedAvg"),
            ScoringStrategy::GoldWeightedAvg => write!(f, "GoldWeightedAvg"),
            ScoringStrategy::DiamondWeightedAvg => write!(f, "DiamondWeightedAvg"),
        }
    }
}

/// Calculates a clan's score using the tier-appropriate strategy.
///
/// # Arguments
/// * `strategy` - The scoring algorithm variant to apply
/// * `member_scores` - List of (user_id, score) tuples
///
/// Returns the calculated score as an i64.
pub fn calculate_score(strategy: ScoringStrategy, member_scores: &[(Uuid, i64)]) -> i64 {
    if member_scores.is_empty() {
        return 0;
    }

    match strategy {
        ScoringStrategy::BronzeSum => member_scores.iter().map(|(_, score)| score).sum(),
        ScoringStrategy::SilverWeightedAvg => {
            let avg = average(member_scores);
            let boosted = (avg as f64 * 1.1).round() as i64;
            boosted
        }
        ScoringStrategy::GoldWeightedAvg => {
            let avg = average(member_scores);
            let with_attendance = (avg as f64 * 1.2).round() as i64;
            with_attendance
        }
        ScoringStrategy::DiamondWeightedAvg => {
            let avg = average(member_scores);
            let with_recency = (avg as f64 * 1.5).round() as i64;
            with_recency
        }
    }
}

/// Computes the arithmetic mean of member scores.
fn average(member_scores: &[(Uuid, i64)]) -> i64 {
    let total: i64 = member_scores.iter().map(|(_, s)| s).sum();
    total / member_scores.len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scores(vals: &[i64]) -> Vec<(Uuid, i64)> {
        vals.iter().map(|&s| (Uuid::new_v4(), s)).collect()
    }

    #[test]
    fn bronze_sum_adds_all_scores() {
        let data = scores(&[10, 20, 30]);
        assert_eq!(calculate_score(ScoringStrategy::BronzeSum, &data), 60);
    }

    #[test]
    fn silver_boosted_average() {
        let data = scores(&[100, 200, 300]);
        // avg = 200, boosted = 200 * 1.1 = 220
        assert_eq!(
            calculate_score(ScoringStrategy::SilverWeightedAvg, &data),
            220
        );
    }

    #[test]
    fn gold_attendance_average() {
        let data = scores(&[100, 200]);
        // avg = 150, with attendance = 150 * 1.2 = 180
        assert_eq!(
            calculate_score(ScoringStrategy::GoldWeightedAvg, &data),
            180
        );
    }

    #[test]
    fn diamond_recency_average() {
        let data = scores(&[50, 50]);
        // avg = 50, with recency = 50 * 1.5 = 75
        assert_eq!(
            calculate_score(ScoringStrategy::DiamondWeightedAvg, &data),
            75
        );
    }

    #[test]
    fn empty_scores_return_zero() {
        let data: Vec<(Uuid, i64)> = vec![];
        assert_eq!(calculate_score(ScoringStrategy::BronzeSum, &data), 0);
        assert_eq!(
            calculate_score(ScoringStrategy::DiamondWeightedAvg, &data),
            0
        );
    }
}
