use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateAchievementRequestDto {
    pub name: String,
    pub milestone_target: i32,
    /// Rarity of the achievement: "Common", "Rare", "Epic", "Legendary".
    pub achievement_type: String,
    /// What event increments this achievement: "QuizComplete", "ReadArticle", "DailyLogin".
    pub trigger_type: String,
    pub reward_points: i32,
}
