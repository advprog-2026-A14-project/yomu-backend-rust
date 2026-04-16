use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::modules::user_sync::domain::entities::shadow_user::ShadowUser;

#[derive(Debug, FromRow)]
pub struct ShadowUserRow {
    pub user_id: Uuid,
    pub total_score: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
}

pub struct UserMapper;

impl UserMapper {
    pub fn from_row(row: &ShadowUserRow) -> Result<ShadowUser, String> {
        let total_score = row.total_score.unwrap_or(0);

        Ok(ShadowUser {
            user_id: row.user_id,
            total_score,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_row_with_valid_data() {
        let row = ShadowUserRow {
            user_id: Uuid::new_v4(),
            total_score: Some(100),
            created_at: Some(Utc::now()),
        };

        let result = UserMapper::from_row(&row);
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.total_score, 100);
    }

    #[test]
    fn from_row_with_null_total_score() {
        let row = ShadowUserRow {
            user_id: Uuid::new_v4(),
            total_score: None,
            created_at: Some(Utc::now()),
        };

        let result = UserMapper::from_row(&row);
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.total_score, 0);
    }

    #[test]
    fn from_row_with_missing_created_at() {
        let row = ShadowUserRow {
            user_id: Uuid::new_v4(),
            total_score: Some(50),
            created_at: None,
        };

        let result = UserMapper::from_row(&row);
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.total_score, 50);
    }
}
