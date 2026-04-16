use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Score {
    base_value: i64,
}

impl Score {
    pub fn new(base_value: i64) -> Self {
        Self { base_value }
    }

    pub fn calculate_with_modifier(base_score: i64, multiplier: f64) -> i64 {
        (base_score as f64 * multiplier).round() as i64
    }

    pub fn base_value(&self) -> i64 {
        self.base_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_with_buff_1_5x() {
        let result = Score::calculate_with_modifier(100, 1.5);
        assert_eq!(result, 150);
    }

    #[test]
    fn test_score_with_debuff_0_8x() {
        let result = Score::calculate_with_modifier(100, 0.8);
        assert_eq!(result, 80);
    }

    #[test]
    fn test_score_no_modifier() {
        let result = Score::calculate_with_modifier(100, 1.0);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_score_new() {
        let score = Score::new(100);
        assert_eq!(score.base_value(), 100);
    }
}
