use crate::modules::user_sync::domain::entities::shadow_user::ShadowUser;
use crate::modules::user_sync::domain::repositories::UserRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserPostgresRepo {
    pool: PgPool,
}

impl UserPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
        .bind(user.user_id)
        .bind(user.total_score)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_query_result) => {
                // ON CONFLICT DO NOTHING means we silently ignore duplicates
                Ok(())
            }
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
}
