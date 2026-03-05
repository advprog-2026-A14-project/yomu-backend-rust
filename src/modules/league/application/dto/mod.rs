use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateClanDto {
    pub name: String,
    pub leader_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct JoinClanDto {
    pub clan_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateScoreDto {
    pub clan_id: Uuid,
    pub user_id: Uuid,
    pub base_score: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub clan_id: Uuid,
    pub clan_name: String,
    pub total_score: i64,
    pub tier: String,
    pub rank: usize,
}

#[derive(Debug, Clone)]
pub struct LeaderboardDto {
    pub entries: Vec<LeaderboardEntry>,
    pub tier: String,
}
