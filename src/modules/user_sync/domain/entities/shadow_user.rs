use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShadowUser {
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    user_id: Uuid,
    #[schema(value_type = i32, example = 0)]
    total_score: i32,
}

impl ShadowUser {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            total_score: 0,
        }
    }

    #[allow(dead_code)]
    pub fn with_id(user_id: Uuid, total_score: i32) -> Self {
        Self {
            user_id,
            total_score,
        }
    }

    pub(crate) fn from_db(user_id: Uuid, total_score: i32) -> Self {
        Self {
            user_id,
            total_score,
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn total_score(&self) -> i32 {
        self.total_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_user_new_creates_with_zero_score() {
        let user_id = Uuid::new_v4();
        let user = ShadowUser::new(user_id);
        assert_eq!(user.user_id(), user_id);
        assert_eq!(user.total_score(), 0);
    }

    #[test]
    fn test_shadow_user_with_id_reconstructs_correctly() {
        let user_id = Uuid::new_v4();
        let score = 1500;
        let user = ShadowUser::with_id(user_id, score);
        assert_eq!(user.user_id(), user_id);
        assert_eq!(user.total_score(), score);
    }

    #[test]
    fn test_shadow_user_with_nil_uuid_is_valid() {
        let nil_uuid = Uuid::nil();
        let user = ShadowUser::new(nil_uuid);
        assert_eq!(user.user_id(), nil_uuid);
        assert_eq!(user.total_score(), 0);
    }

    #[test]
    fn test_shadow_user_boundary_values() {
        let max_score = i32::MAX;
        let user = ShadowUser::with_id(Uuid::nil(), max_score);
        assert_eq!(user.total_score(), max_score);

        let min_score = i32::MIN;
        let user2 = ShadowUser::with_id(Uuid::nil(), min_score);
        assert_eq!(user2.total_score(), min_score);
    }

    #[test]
    fn test_shadow_user_clone_is_independent() {
        let user_id = Uuid::new_v4();
        let user = ShadowUser::new(user_id);
        let cloned = user.clone();
        assert_eq!(user.user_id(), cloned.user_id());
        assert_eq!(user.total_score(), cloned.total_score());
    }
}
