use crate::modules::league::domain::entities::clan_join_request::ClanJoinRequest;
use crate::modules::league::domain::entities::clan_join_request::RequestStatus;
use crate::modules::league::domain::repositories::ClanJoinRequestRepository;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct ClanJoinRequestPostgresRepo {
    pool: PgPool,
}

impl ClanJoinRequestPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClanJoinRequestRepository for ClanJoinRequestPostgresRepo {
    async fn create_request(&self, request: &ClanJoinRequest) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO clan_join_requests (id, clan_id, user_id, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(request.id())
        .bind(request.clan_id())
        .bind(request.user_id())
        .bind(request.status().to_string())
        .bind(request.created_at())
        .bind(request.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }

    async fn get_pending_requests_by_clan(&self, clan_id: Uuid) -> Result<Vec<ClanJoinRequest>, AppError> {
        let rows = sqlx::query(
            "SELECT id, clan_id, user_id, status, created_at, updated_at FROM clan_join_requests WHERE clan_id = $1 AND status = 'Pending'",
        )
        .bind(clan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        let requests = rows
            .iter()
            .map(|row| {
                let id: Uuid = row.get("id");
                let clan_id: Uuid = row.get("clan_id");
                let user_id: Uuid = row.get("user_id");
                let status_str: String = row.get("status");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
                ClanJoinRequest::with_id(id, clan_id, user_id, RequestStatus::from_str(&status_str), created_at, updated_at)
            })
            .collect();

        Ok(requests)
    }

    async fn get_request_by_id(&self, request_id: Uuid) -> Result<Option<ClanJoinRequest>, AppError> {
        let row = sqlx::query(
            "SELECT id, clan_id, user_id, status, created_at, updated_at FROM clan_join_requests WHERE id = $1",
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        match row {
            Some(r) => {
                let id: Uuid = r.get("id");
                let clan_id: Uuid = r.get("clan_id");
                let user_id: Uuid = r.get("user_id");
                let status_str: String = r.get("status");
                let created_at: chrono::DateTime<chrono::Utc> = r.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = r.get("updated_at");
                Ok(Some(ClanJoinRequest::with_id(
                    id, clan_id, user_id, RequestStatus::from_str(&status_str), created_at, updated_at,
                )))
            }
            None => Ok(None),
        }
    }

    async fn get_pending_request_by_user(
        &self,
        user_id: Uuid,
        clan_id: Uuid,
    ) -> Result<Option<ClanJoinRequest>, AppError> {
        let row = sqlx::query(
            "SELECT id, clan_id, user_id, status, created_at, updated_at FROM clan_join_requests WHERE user_id = $1 AND clan_id = $2 AND status = 'Pending'",
        )
        .bind(user_id)
        .bind(clan_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        match row {
            Some(r) => {
                let id: Uuid = r.get("id");
                let cid: Uuid = r.get("clan_id");
                let uid: Uuid = r.get("user_id");
                let status_str: String = r.get("status");
                let created_at: chrono::DateTime<chrono::Utc> = r.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = r.get("updated_at");
                Ok(Some(ClanJoinRequest::with_id(
                    id, cid, uid, RequestStatus::from_str(&status_str), created_at, updated_at,
                )))
            }
            None => Ok(None),
        }
    }

    async fn has_pending_request(&self, user_id: Uuid) -> Result<bool, AppError> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM clan_join_requests WHERE user_id = $1 AND status = 'Pending'")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(count > 0)
    }

    async fn update_request_status(&self, request_id: Uuid, status: &RequestStatus) -> Result<(), AppError> {
        let now = chrono::Utc::now();
        sqlx::query("UPDATE clan_join_requests SET status = $1, updated_at = $2 WHERE id = $3")
            .bind(status.to_string())
            .bind(now)
            .bind(request_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(())
    }

    async fn delete_request(&self, request_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM clan_join_requests WHERE id = $1")
            .bind(request_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(())
    }

    async fn delete_pending_requests_by_user(&self, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM clan_join_requests WHERE user_id = $1 AND status = 'Pending'")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;
        Ok(())
    }
}
