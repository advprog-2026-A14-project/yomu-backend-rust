use async_trait::async_trait;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::modules::gamification::domain::entities::achievement::{
    Achievement, AchievementType, UserAchievement,
};
use crate::modules::gamification::domain::ports::achievement_repository::AchievementRepository;

pub struct PostgresAchievementRepository {
    pub pool: PgPool,
}

impl PostgresAchievementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AchievementRepository for PostgresAchievementRepository {
    async fn get_achievement_by_id(&self, id: Uuid) -> Result<Option<Achievement>, String> {
        let record = sqlx::query!(
            r#"
            SELECT id, name, milestone_target, achievement_type, reward_points 
            FROM achievements 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error (get_achievement_by_id): {}", e))?;

        match record {
            Some(row) => {
                // Mapping manual string dari database kembali ke Enum Rust
                let ach_type = match row.achievement_type.as_str() {
                    "Rare" => AchievementType::Rare,
                    "Epic" => AchievementType::Epic,
                    "Legendary" => AchievementType::Legendary,
                    _ => AchievementType::Common, // Default
                };

                let achievement = Achievement::new(
                    row.id,
                    row.name,
                    row.milestone_target,
                    ach_type,
                    row.reward_points,
                )
                .map_err(|e| e.to_string())?;

                Ok(Some(achievement))
            }
            None => Ok(None),
        }
    }

    async fn get_user_achievements(&self, user_id: Uuid) -> Result<Vec<UserAchievement>, String> {
        let records = sqlx::query!(
            r#"
            SELECT user_id, achievement_id, current_progress, is_completed, is_shown_on_profile, completed_at 
            FROM user_achievements 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error (get_user_achievements): {}", e))?;

        let mut achievements = Vec::new();

        for row in records {
            // Karena field di struct UserAchievement bersifat 'pub',
            // kita bisa langsung merekonstruksi state masa lalunya dari database.
            let mut user_ach = UserAchievement::new(row.user_id, row.achievement_id);
            user_ach.current_progress = row.current_progress;
            user_ach.is_completed = row.is_completed;
            user_ach.is_shown_on_profile = row.is_shown_on_profile;
            user_ach.completed_at = row.completed_at;

            achievements.push(user_ach);
        }

        Ok(achievements)
    }

    async fn save_user_achievement(
        &self,
        user_achievement: &UserAchievement,
    ) -> Result<(), String> {
        // UPSERT: Insert jika baru pertama kali dapat progres, Update jika sudah ada
        sqlx::query!(
            r#"
            INSERT INTO user_achievements 
            (user_id, achievement_id, current_progress, is_completed, is_shown_on_profile, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, achievement_id) 
            DO UPDATE SET 
                current_progress = EXCLUDED.current_progress,
                is_completed = EXCLUDED.is_completed,
                is_shown_on_profile = EXCLUDED.is_shown_on_profile,
                completed_at = EXCLUDED.completed_at
            "#,
            user_achievement.user_id(),
            user_achievement.achievement_id(),
            user_achievement.current_progress(),
            user_achievement.is_completed(),
            user_achievement.is_shown_on_profile(),
            user_achievement.completed_at() // Ini Option<DateTime<Utc>>, sqlx bisa handle otomatis ke format NULL di DB
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Gagal menyimpan progres pencapaian: {}", e))?;

        Ok(())
    }

    async fn add_user_score(&self, user_id: Uuid, points: i32) -> Result<(), String> {
        // Query ini persis sama dengan yang ada di mission_repository,
        // tapi ditaruh di sini agar AchievementUseCase tetap independen.
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
        .map_err(|e| format!("Gagal menambah skor user dari pencapaian: {}", e))?;

        Ok(())
    }

    async fn get_all_achievements(&self) -> Result<Vec<Achievement>, String> {
        let records = sqlx::query!(
            r#"
            SELECT id, name, milestone_target, achievement_type, reward_points 
            FROM achievements
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error (get_all_achievements): {}", e))?;

        let mut achievements = Vec::new();
        for row in records {
            let ach_type = match row.achievement_type.as_str() {
                "Rare" => AchievementType::Rare,
                "Epic" => AchievementType::Epic,
                "Legendary" => AchievementType::Legendary,
                _ => AchievementType::Common,
            };

            if let Ok(achievement) = Achievement::new(
                row.id,
                row.name,
                row.milestone_target,
                ach_type,
                row.reward_points,
            ) {
                achievements.push(achievement);
            }
        }

        Ok(achievements)
    }

    async fn get_achievements_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Achievement>, String> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let records = sqlx::query!(
            r#"
            SELECT id, name, milestone_target, achievement_type, reward_points 
            FROM achievements 
            WHERE id = ANY($1)
            "#,
            ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error (get_achievements_by_ids): {}", e))?;

        let mut achievements = Vec::new();
        for row in records {
            let ach_type = match row.achievement_type.as_str() {
                "Rare" => AchievementType::Rare,
                "Epic" => AchievementType::Epic,
                "Legendary" => AchievementType::Legendary,
                _ => AchievementType::Common,
            };

            if let Ok(achievement) = Achievement::new(
                row.id,
                row.name,
                row.milestone_target,
                ach_type,
                row.reward_points,
            ) {
                achievements.push(achievement);
            }
        }

        Ok(achievements)
    }
}
