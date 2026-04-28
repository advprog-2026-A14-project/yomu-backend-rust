use crate::modules::league::domain::entities::clan::Clan;
use crate::modules::league::domain::entities::clan::ClanTier;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::shared::domain::base_error::AppError;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct ClanPostgresRepo {
    pool: PgPool,
}

impl ClanPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClanRepository for ClanPostgresRepo {
    /// Inserts a new clan record into the clans table.
    async fn create_clan(&self, clan: &Clan) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO clans (id, name, leader_id, tier, total_score, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(clan.id())
        .bind(clan.name())
        .bind(clan.leader_id())
        .bind(clan.tier().to_string())
        .bind(clan.total_score())
        .bind(clan.created_at())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }

    /// Retrieves a clan by ID from PostgreSQL.
    ///
    /// Maps database tier string ("Bronze", "Silver", etc.) to ClanTier enum.
    /// Returns None if clan does not exist.
    async fn get_clan_by_id(&self, clan_id: Uuid) -> Result<Option<Clan>, AppError> {
        let row = sqlx::query_as::<_, ClanRow>(
            "SELECT id, name, leader_id, tier, total_score::int8, created_at FROM clans WHERE id = $1",
        )
        .bind(clan_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        match row {
            Some(r) => {
                let tier = match r.tier.as_str() {
                    "Silver" => ClanTier::Silver,
                    "Gold" => ClanTier::Gold,
                    "Diamond" => ClanTier::Diamond,
                    _ => ClanTier::Bronze,
                };
                Ok(Some(Clan::with_id(
                    r.id,
                    r.name,
                    r.leader_id,
                    tier,
                    r.total_score,
                    r.created_at,
                )))
            }
            None => Ok(None),
        }
    }

    /// Inserts a new member into the clan_members table.
    async fn add_member(&self, member: &ClanMember) -> Result<(), AppError> {
        sqlx::query("INSERT INTO clan_members (clan_id, user_id, joined_at) VALUES ($1, $2, $3)")
            .bind(member.clan_id())
            .bind(member.user_id())
            .bind(member.joined_at())
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }

    /// Retrieves all members for a clan.
    ///
    /// Role is determined by comparing user_id with the clan's leader_id.
    /// Uses a single JOIN query to avoid N+1 problem.
    async fn get_members_by_clan_id(&self, clan_id: Uuid) -> Result<Vec<ClanMember>, AppError> {
        // Single JOIN query to get both clan leader_id and members
        let rows = sqlx::query(
            r#"
            SELECT
                cm.clan_id,
                cm.user_id,
                cm.joined_at,
                c.leader_id
            FROM clan_members cm
            INNER JOIN clans c ON cm.clan_id = c.id
            WHERE cm.clan_id = $1
            "#,
        )
        .bind(clan_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        // Handle case where clan doesn't exist (empty result from JOIN)
        if rows.is_empty() {
            return Ok(vec![]);
        }

        let members = rows
            .iter()
            .map(|row| {
                let user_id: Uuid = row.get("user_id");
                let leader_id: Uuid = row.get("leader_id");
                let joined_at: chrono::DateTime<chrono::Utc> = row.get("joined_at");

                // Determine role based on whether user is the leader
                let role = if user_id == leader_id {
                    crate::modules::league::domain::entities::clan_member::MemberRole::Leader
                } else {
                    crate::modules::league::domain::entities::clan_member::MemberRole::Member
                };

                ClanMember::with_joined_at(clan_id, user_id, role, joined_at)
            })
            .collect();

        Ok(members)
    }

    /// Checks if a user is already a member of any clan.
    ///
    /// Used for validation before creating or joining a clan.
    async fn is_user_in_any_clan(&self, user_id: Uuid) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clan_members WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(count > 0)
    }

    async fn get_user_clan_id(&self, user_id: Uuid) -> Result<Option<Uuid>, AppError> {
        let row = sqlx::query("SELECT clan_id FROM clan_members WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.get("clan_id"))),
            None => Ok(None),
        }
    }

    /// Retrieves user tier info (clan_id, clan_name, tier) using a single JOIN query.
    async fn get_user_tier_info(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(Uuid, String, ClanTier)>, AppError> {
        let row = sqlx::query_as::<_, UserTierRow>(
            r#"
            SELECT c.id, c.name, c.tier
            FROM clans c
            INNER JOIN clan_members cm ON c.id = cm.clan_id
            WHERE cm.user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServer(e.to_string()))?;

        match row {
            Some(r) => {
                let tier = match r.tier.as_str() {
                    "Silver" => ClanTier::Silver,
                    "Gold" => ClanTier::Gold,
                    "Diamond" => ClanTier::Diamond,
                    _ => ClanTier::Bronze,
                };
                Ok(Some((r.id, r.name, tier)))
            }
            None => Ok(None),
        }
    }

    async fn add_score(&self, clan_id: Uuid, score: i64) -> Result<(), AppError> {
        sqlx::query("UPDATE clans SET total_score = total_score + $1 WHERE id = $2")
            .bind(score)
            .bind(clan_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServer(e.to_string()))?;

        Ok(())
    }
}

// Helper struct for sqlx::query_as!
#[derive(sqlx::FromRow)]
struct ClanRow {
    id: Uuid,
    name: String,
    leader_id: Uuid,
    tier: String,
    total_score: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct UserTierRow {
    id: Uuid,
    name: String,
    tier: String,
}
