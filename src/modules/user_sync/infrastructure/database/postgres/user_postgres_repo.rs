use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::modules::user_sync::domain::entities::shadow_user::ShadowUser;
use crate::modules::user_sync::domain::repositories::UserRepository;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Debug, FromRow)]
pub struct ShadowUserRow {
    pub user_id: Uuid,
    pub total_score: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
}

pub struct UserPostgresRepo {
    pool: PgPool,
}

impl UserPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row_to_shadow_user(row: &ShadowUserRow) -> Result<ShadowUser, AppError> {
        let total_score = row.total_score.unwrap_or(0);
        Ok(ShadowUser::with_id(row.user_id, total_score))
    }
}

#[async_trait]
impl UserRepository for UserPostgresRepo {
    async fn insert_shadow_user(
        &self,
        user: &ShadowUser,
    ) -> Result<(), crate::shared::domain::base_error::AppError> {
        let result = sqlx::query(
            "INSERT INTO shadow_users (user_id, total_score, created_at) VALUES ($1, $2, NOW()) ON CONFLICT (user_id) DO NOTHING"
        )
        .bind(user.user_id())
        .bind(user.total_score())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_query_result) => Ok(()),
            Err(e) => Err(crate::shared::domain::base_error::AppError::InternalServer(
                e.to_string(),
            )),
        }
    }

    async fn exists_shadow_user(
        &self,
        user_id: Uuid,
    ) -> Result<bool, crate::shared::domain::base_error::AppError> {
        let result =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM shadow_users WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await;

        match result {
            Ok(count) => Ok(count > 0),
            Err(e) => Err(crate::shared::domain::base_error::AppError::InternalServer(
                e.to_string(),
            )),
        }
    }

    async fn check_exists(&self, user_id: Uuid) -> bool {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM shadow_users WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map(|count| count > 0)
            .unwrap_or(false)
    }

    async fn get_shadow_user(&self, user_id: Uuid) -> Result<Option<ShadowUser>, AppError> {
        let result = sqlx::query_as::<_, ShadowUserRow>(
            "SELECT user_id, total_score, created_at FROM shadow_users WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(Self::map_row_to_shadow_user(&row)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::InternalServer(e.to_string())),
        }
    }

    async fn update_total_score(&self, user_id: Uuid, score_to_add: i32) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE shadow_users SET total_score = total_score + $1 WHERE user_id = $2",
        )
        .bind(score_to_add)
        .bind(user_id)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_query_result) => Ok(()),
            Err(e) => Err(AppError::InternalServer(e.to_string())),
        }
    }
}
