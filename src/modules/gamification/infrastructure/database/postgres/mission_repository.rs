use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;

use crate::modules::gamification::domain::ports::mission_repository::MissionRepository;
use crate::modules::gamification::domain::entities::mission::{DailyMission, UserMission};

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
    async fn get_user_mission(&self, user_id: Uuid, mission_id: Uuid) -> Result<Option<UserMission>, String> {
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
                // Injeksi state dari database secara manual
                mission.add_progress(row.current_progress, row.current_progress); 
                if row.is_claimed {
                    let _ = mission.claim_reward(row.current_progress); 
                }
                Ok(Some(mission))
            },
            None => Ok(None)
        }
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
            WHERE id = $2
            "#,
            points,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Gagal menambah skor user: {}", e))?;

        Ok(())
    }

    async fn get_active_missions_by_date(&self, _date: NaiveDate) -> Result<Vec<DailyMission>, String> {
        let records = sqlx::query!(
            r#"
            SELECT id, description, target_count, target_date, reward_points 
            FROM daily_missions 
            WHERE target_date = $1
            "#,
            date
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error (get_active_missions): {}", e))?;

        // Mengubah Vec dari row database menjadi Vec<DailyMission>
        let mut missions = Vec::new();
        for row in records {
            if let Ok(mission) = DailyMission::new(
                row.id, 
                row.description, 
                row.target_count, 
                row.target_date, 
                row.reward_points
            ) {
                missions.push(mission);
            }
        }

        Ok(missions)
    }

    async fn get_daily_mission_by_id(&self, _id: Uuid) -> Result<Option<DailyMission>, String> {
       let record = sqlx::query!(
            r#"
            SELECT id, description, target_count, target_date, reward_points 
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
                let mission = DailyMission::new(
                    row.id, 
                    row.description, 
                    row.target_count, 
                    row.target_date, 
                    row.reward_points
                ).map_err(|e| e.to_string())?;
                Ok(Some(mission))
            },
            None => Ok(None)
        }
    }
}