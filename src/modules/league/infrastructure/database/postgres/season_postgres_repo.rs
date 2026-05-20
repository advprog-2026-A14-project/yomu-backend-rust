use crate::modules::league::domain::entities::clan::ClanTier;
use crate::modules::league::domain::entities::season::Season;
use crate::modules::league::domain::repositories::season_repository::SeasonRepository;
use crate::modules::league::domain::repositories::season_repository::SeasonResultRow;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct SeasonPostgresRepo {
    pool: PgPool,
}

impl SeasonPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SeasonRepository for SeasonPostgresRepo {
    async fn create_season(&self, season: &Season) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO seasons (id, name, tier, starts_at, ends_at, is_active) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(season.id())
        .bind(season.name())
        .bind(season.tier().to_string())
        .bind(season.starts_at())
        .bind(season.ends_at())
        .bind(season.is_active())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }

    async fn get_active_season(&self, tier: &ClanTier) -> Result<Option<Season>, AppError> {
        let row = sqlx::query_as::<_, SeasonRow>(
            "SELECT id, name, tier, starts_at, ends_at, is_active FROM seasons WHERE tier = $1 AND is_active = true"
        )
        .bind(tier.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        match row {
            Some(r) => {
                let tier = parse_tier(&r.tier);
                Ok(Some(Season::with_id(
                    r.id,
                    r.name,
                    tier,
                    r.starts_at,
                    r.ends_at,
                    r.is_active,
                )))
            }
            None => Ok(None),
        }
    }

    async fn get_season_by_id(&self, season_id: Uuid) -> Result<Option<Season>, AppError> {
        let row = sqlx::query_as::<_, SeasonRow>(
            "SELECT id, name, tier, starts_at, ends_at, is_active FROM seasons WHERE id = $1",
        )
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        match row {
            Some(r) => {
                let tier = parse_tier(&r.tier);
                Ok(Some(Season::with_id(
                    r.id,
                    r.name,
                    tier,
                    r.starts_at,
                    r.ends_at,
                    r.is_active,
                )))
            }
            None => Ok(None),
        }
    }

    async fn get_season_results(
        &self,
        tier: &ClanTier,
        _season_id: Uuid,
    ) -> Result<Vec<SeasonResultRow>, AppError> {
        let rows = sqlx::query_as::<_, ClanScoreRow>(
            "SELECT id, name, total_score FROM clans WHERE tier = $1 ORDER BY total_score DESC",
        )
        .bind(tier.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        let results: Vec<SeasonResultRow> = rows
            .into_iter()
            .enumerate()
            .map(|(idx, row)| {
                let rank = (idx + 1) as i64;
                (row.id, row.name, rank, row.total_score)
            })
            .collect();

        Ok(results)
    }

    async fn mark_season_ended(&self, season_id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE seasons SET is_active = false WHERE id = $1")
            .bind(season_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }

    async fn update_clan_tier(&self, clan_id: Uuid, new_tier: &ClanTier) -> Result<(), AppError> {
        sqlx::query("UPDATE clans SET tier = $1 WHERE id = $2")
            .bind(new_tier.to_string())
            .bind(clan_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }
}

fn parse_tier(tier_str: &str) -> ClanTier {
    match tier_str {
        "Silver" => ClanTier::Silver,
        "Gold" => ClanTier::Gold,
        "Diamond" => ClanTier::Diamond,
        _ => ClanTier::Bronze,
    }
}

#[derive(sqlx::FromRow)]
struct SeasonRow {
    id: Uuid,
    name: String,
    tier: String,
    starts_at: chrono::DateTime<chrono::Utc>,
    ends_at: chrono::DateTime<chrono::Utc>,
    is_active: bool,
}

#[derive(sqlx::FromRow)]
struct ClanScoreRow {
    id: Uuid,
    name: String,
    total_score: i64,
}
