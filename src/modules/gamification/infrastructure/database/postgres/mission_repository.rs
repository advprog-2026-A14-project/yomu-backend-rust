use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::gamification::domain::entities::daily_mission::{DailyMission, MissionType};
use crate::modules::gamification::domain::entities::user_mission::UserMission;
use crate::modules::gamification::domain::repositories::mission_repository::MissionRepository;

pub struct PostgresMissionRepository {
    pub pool: PgPool,
}

impl PostgresMissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MissionRepository for PostgresMissionRepository {
    async fn get_user_mission(
        &self,
        user_id: Uuid,
        mission_id: Uuid,
    ) -> Result<Option<UserMission>, String> {
        let record = sqlx::query!(
            r#"
            SELECT user_id, mission_id, current_progress, is_claimed
            FROM user_missions
            WHERE user_id = $1 AND mission_id = $2
            "#,
            user_id,
            mission_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        // Mapping dari hasil row database ke Entity Rust
        match record {
            Some(row) => {
                let mut mission = UserMission::new(row.user_id, row.mission_id);
                mission.current_progress = row.current_progress;
                mission.is_claimed = row.is_claimed;
                Ok(Some(mission))
            }
            None => Ok(None),
        }
    }

    async fn get_user_missions_batch(
        &self,
        user_id: Uuid,
        mission_ids: Vec<Uuid>,
    ) -> Result<Vec<UserMission>, String> {
        if mission_ids.is_empty() {
            return Ok(Vec::new());
        }

        let records = sqlx::query!(
            r#"
            SELECT user_id, mission_id, current_progress, is_claimed
            FROM user_missions
            WHERE user_id = $1 AND mission_id = ANY($2)
            "#,
            user_id,
            &mission_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        let mut user_missions = Vec::new();
        for row in records {
            let mut mission = UserMission::new(row.user_id, row.mission_id);
            mission.current_progress = row.current_progress;
            mission.is_claimed = row.is_claimed;
            user_missions.push(mission);
        }

        Ok(user_missions)
    }

    async fn save_user_mission(&self, user_mission: &UserMission) -> Result<(), String> {
        sqlx::query!(
            r#"
            INSERT INTO user_missions (user_id, mission_id, current_progress, is_claimed)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, mission_id) 
            DO UPDATE SET 
                current_progress = EXCLUDED.current_progress,
                is_claimed = EXCLUDED.is_claimed
            "#,
            user_mission.user_id(),
            user_mission.mission_id(),
            user_mission.current_progress(),
            user_mission.is_claimed()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Gagal menyimpan progres misi: {}", e))?;

        Ok(())
    }

    async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String> {
        sqlx::query!(
            r#"
            UPDATE engine_users 
            SET total_score = total_score + $1 
            WHERE user_id = $2
            "#,
            points,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Gagal menambah skor user: {}", e))?;

        Ok(())
    }

    async fn get_active_missions_by_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<DailyMission>, String> {
        let records = sqlx::query!(
            r#"
            SELECT id, description, target_count, date, reward_points, mission_type
            FROM daily_missions 
            WHERE date = $1
            "#,
            date
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error (get_active_missions): {}", e))?;

        // Mengubah Vec dari row database menjadi Vec<DailyMission>
        let mut missions = Vec::new();
        for row in records {
            let m_type = match row.mission_type.as_str() {
                "Quiz" => MissionType::Quiz,
                "DailyLogin" => MissionType::DailyLogin,
                _ => MissionType::ReadArticle,
            };

            if let Ok(mission) = DailyMission::new(
                row.id,
                row.description,
                row.target_count,
                row.date,
                row.reward_points,
                m_type,
            ) {
                missions.push(mission);
            }
        }

        Ok(missions)
    }

    async fn get_daily_mission_by_id(&self, id: Uuid) -> Result<Option<DailyMission>, String> {
        let record = sqlx::query!(
            r#"
            SELECT id, description, target_count, date, reward_points, mission_type 
            FROM daily_missions 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error (get_daily_mission): {}", e))?;

        match record {
            Some(row) => {
                let m_type = match row.mission_type.as_str() {
                    "Quiz" => MissionType::Quiz,
                    "DailyLogin" => MissionType::DailyLogin,
                    _ => MissionType::ReadArticle,
                };

                let mission = DailyMission::new(
                    row.id,
                    row.description,
                    row.target_count,
                    row.date,
                    row.reward_points,
                    m_type,
                )
                .map_err(|e| e.to_string())?;
                Ok(Some(mission))
            }
            None => Ok(None),
        }
    }

    async fn create_daily_mission(&self, mission: &DailyMission) -> Result<(), String> {
        sqlx::query!(
            r#"
            INSERT INTO daily_missions (id, description, target_count, date, reward_points, mission_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            mission.id(),
            mission.description(),
            mission.target_count(),
            mission.date(),
            mission.reward_points(),
            format!("{:?}", mission.mission_type())
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Gagal membuat misi harian: {}", e))?;

        Ok(())
    }

    async fn update_daily_mission(&self, mission: &DailyMission) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            UPDATE daily_missions
            SET
                description = $2,
                target_count = $3,
                date = $4,
                reward_points = $5,
                mission_type = $6
            WHERE id = $1
            "#,
            mission.id(),
            mission.description(),
            mission.target_count(),
            mission.date(),
            mission.reward_points(),
            format!("{:?}", mission.mission_type())
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Gagal memperbarui misi harian: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Data misi harian tidak ditemukan di sistem.".to_string());
        }

        Ok(())
    }

    async fn delete_daily_mission(&self, id: Uuid) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM daily_missions
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Gagal menghapus misi harian: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Data misi harian tidak ditemukan di sistem.".to_string());
        }

        Ok(())
}
}
