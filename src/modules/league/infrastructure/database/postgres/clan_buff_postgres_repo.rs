use crate::modules::league::domain::entities::clan_buff::ClanBuff;
use crate::modules::league::domain::repositories::ClanBuffRepository;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct ClanBuffPostgresRepo {
    pool: PgPool,
}

impl ClanBuffPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClanBuffRepository for ClanBuffPostgresRepo {
    async fn activate_buff(&self, buff: &ClanBuff) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO clan_buffs (id, clan_id, buff_name, multiplier, is_active, is_debuff, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (clan_id, buff_name)
            DO UPDATE SET
                multiplier = EXCLUDED.multiplier,
                is_active = EXCLUDED.is_active,
                is_debuff = EXCLUDED.is_debuff,
                expires_at = EXCLUDED.expires_at
            "#,
        )
        .bind(buff.id())
        .bind(buff.clan_id())
        .bind(buff.buff_name())
        .bind(buff.multiplier())
        .bind(buff.is_active())
        .bind(buff.is_debuff())
        .bind(buff.expires_at())
        .bind(buff.created_at())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }

    async fn deactivate_buffs_for_clan(&self, clan_id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE clan_buffs SET is_active = false WHERE clan_id = $1")
            .bind(clan_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }

    async fn get_active_buffs(&self, clan_id: Uuid) -> Result<Vec<ClanBuff>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, clan_id, buff_name, multiplier::float8, is_active, is_debuff, expires_at, created_at
            FROM clan_buffs
            WHERE clan_id = $1 AND is_active = true AND expires_at > NOW()
            "#,
        )
        .bind(clan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(rows.iter().map(row_to_clan_buff).collect())
    }

    async fn get_buff_by_name(
        &self,
        clan_id: Uuid,
        name: &str,
    ) -> Result<Option<ClanBuff>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, clan_id, buff_name, multiplier::float8, is_active, is_debuff, expires_at, created_at
            FROM clan_buffs
            WHERE clan_id = $1 AND buff_name = $2
            "#,
        )
        .bind(clan_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(row.as_ref().map(row_to_clan_buff))
    }

    async fn get_avg_accuracy_for_members(&self, user_ids: &[Uuid]) -> Result<f64, AppError> {
        if user_ids.is_empty() {
            return Ok(0.0);
        }
        let total: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(AVG(qh.accuracy), 0.0)
            FROM quiz_history qh
            WHERE qh.user_id = ANY($1)
              AND qh.completed_at > NOW() - INTERVAL '7 days'
            "#,
        )
        .bind(user_ids)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(total)
    }

    async fn get_mission_completion_rate_for_clan(&self, clan_id: Uuid) -> Result<f64, AppError> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clan_members WHERE clan_id = $1")
            .bind(clan_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;

        if total == 0 {
            return Ok(0.0);
        }

        let completed: f64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT um.user_id)::float
            FROM user_missions um
            JOIN daily_missions dm ON um.mission_id = dm.id
            JOIN clan_members cm ON um.user_id = cm.user_id
            WHERE dm.date = CURRENT_DATE
              AND um.is_claimed = true
              AND cm.clan_id = $1
            "#,
        )
        .bind(clan_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(completed / total as f64)
    }

    async fn deactivate_buff_by_name(
        &self,
        clan_id: Uuid,
        buff_name: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE clan_buffs SET is_active = false WHERE clan_id = $1 AND buff_name = $2",
        )
        .bind(clan_id)
        .bind(buff_name)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(())
    }

    async fn get_avg_quiz_score_for_members(&self, user_ids: &[Uuid]) -> Result<i64, AppError> {
        let score: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(AVG(qh.score::float), 0.0)
            FROM quiz_history qh
            WHERE qh.user_id = ANY($1)
              AND qh.completed_at > NOW() - INTERVAL '30 days'
            "#,
        )
        .bind(user_ids)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(score.round() as i64)
    }
}

fn row_to_clan_buff(row: &sqlx::postgres::PgRow) -> ClanBuff {
    ClanBuff::new(
        row.get("id"),
        row.get("clan_id"),
        row.get("buff_name"),
        row.get("multiplier"),
        row.get("is_active"),
        row.get("is_debuff"),
        row.get("expires_at"),
        row.get("created_at"),
    )
}
