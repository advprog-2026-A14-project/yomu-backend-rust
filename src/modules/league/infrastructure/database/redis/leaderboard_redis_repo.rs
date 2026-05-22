use crate::modules::league::application::dto::LeaderboardEntry;
use crate::modules::league::domain::repositories::LeaderboardCache;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use uuid::Uuid;

pub struct LeaderboardRedisRepo {
    conn: MultiplexedConnection,
}

impl LeaderboardRedisRepo {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }

    fn get_key(&self, tier: &str) -> String {
        format!("leaderboard:{}", tier)
    }
}

#[async_trait]
impl LeaderboardCache for LeaderboardRedisRepo {
    async fn update_clan_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError> {
        let mut con = self.conn.clone();
        let key = self.get_key("global");
        let _: String = redis::cmd("ZINCRBY")
            .arg(&key)
            .arg(score)
            .arg(clan_id.to_string())
            .query_async(&mut con)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(())
    }

    async fn add_clan_to_tier(&self, clan_id: Uuid, tier: &str) -> Result<(), AppError> {
        let mut con = self.conn.clone();
        let key = self.get_key(tier);
        let _: String = redis::cmd("ZADD")
            .arg(&key)
            .arg(0)
            .arg(clan_id.to_string())
            .query_async(&mut con)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(())
    }

    async fn get_top_clans(
        &self,
        tier: &str,
        limit: usize,
    ) -> Result<Vec<LeaderboardEntry>, AppError> {
        let mut con = self.conn.clone();
        let key = self.get_key(tier);
        let results: Vec<(String, String)> = redis::cmd("ZREVRANGE")
            .arg(&key)
            .arg(0)
            .arg(limit - 1)
            .arg("WITHSCORES")
            .query_async(&mut con)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;
        let entries: Vec<LeaderboardEntry> = results
            .iter()
            .enumerate()
            .map(|(idx, (clan_id_str, score_str))| {
                let clan_id = Uuid::parse_str(clan_id_str).unwrap_or_else(|_| Uuid::nil());
                let total_score: i64 = score_str.parse().unwrap_or(0);
                LeaderboardEntry {
                    clan_id,
                    clan_name: format!("Clan {}", &clan_id_str[..8.min(clan_id_str.len())]),
                    total_score,
                    tier: tier.to_string(),
                    rank: idx + 1,
                }
            })
            .collect();
        Ok(entries)
    }

    async fn get_clan_score(&self, clan_id: Uuid) -> Result<Option<i64>, AppError> {
        let mut con = self.conn.clone();
        let key = self.get_key("global");
        let score: Option<f64> = redis::cmd("ZSCORE")
            .arg(&key)
            .arg(clan_id.to_string())
            .query_async(&mut con)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(score.map(|s| s as i64))
    }

    async fn remove_clan_from_leaderboard(&self, clan_id: Uuid) -> Result<(), AppError> {
        let mut con = self.conn.clone();
        let key = self.get_key("global");
        let _: () = redis::cmd("ZREM")
            .arg(&key)
            .arg(clan_id.to_string())
            .query_async(&mut con)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(())
    }
}
