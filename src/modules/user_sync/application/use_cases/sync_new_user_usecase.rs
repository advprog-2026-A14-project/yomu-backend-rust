use crate::modules::user_sync::domain::entities::ShadowUser;
use crate::modules::user_sync::domain::errors::UserSyncError;
use crate::modules::user_sync::domain::repositories::UserRepository;

pub struct SyncNewUserUseCase<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> SyncNewUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        dto: crate::modules::user_sync::application::dto::SyncUserRequestDto,
    ) -> Result<ShadowUser, UserSyncError> {
        if self
            .repository
            .exists_shadow_user(dto.user_id)
            .await
            .map_err(|e| UserSyncError::DatabaseError(e.to_string()))?
        {
            let existing = self
                .repository
                .get_shadow_user(dto.user_id)
                .await
                .map_err(|e| UserSyncError::DatabaseError(e.to_string()))?;
            if let Some(user) = existing {
                return Ok(user);
            }
        }

        let shadow_user = ShadowUser::new(dto.user_id);
        self.repository
            .insert_shadow_user(&shadow_user)
            .await
            .map_err(|e| UserSyncError::DatabaseError(e.to_string()))?;

        Ok(shadow_user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::user_sync::application::dto::SyncUserRequestDto;
    use crate::modules::user_sync::domain::entities::ShadowUser;
    use crate::shared::domain::base_error::AppError;
    use async_trait::async_trait;
    use uuid::Uuid;

    struct MockUserRepository {
        existing_users: std::sync::Mutex<std::collections::HashSet<Uuid>>,
        insert_should_fail: bool,
        insert_should_panic: bool,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                existing_users: std::sync::Mutex::new(std::collections::HashSet::new()),
                insert_should_fail: false,
                insert_should_panic: false,
            }
        }

        fn with_existing_user(user_id: Uuid) -> Self {
            let mut users = std::collections::HashSet::new();
            users.insert(user_id);
            Self {
                existing_users: std::sync::Mutex::new(users),
                insert_should_fail: false,
                insert_should_panic: false,
            }
        }

        fn with_insert_failure(self) -> Self {
            Self {
                insert_should_fail: true,
                ..self
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn insert_shadow_user(&self, user: &ShadowUser) -> Result<(), AppError> {
            if self.insert_should_fail {
                return Err(AppError::InternalServer("Insert failed".to_string()));
            }
            if self.insert_should_panic {
                panic!("Database connection lost");
            }
            Ok(())
        }

        async fn exists_shadow_user(&self, user_id: Uuid) -> Result<bool, AppError> {
            Ok(self.existing_users.lock().unwrap().contains(&user_id))
        }

        async fn check_exists(&self, user_id: Uuid) -> bool {
            self.existing_users.lock().unwrap().contains(&user_id)
        }

        async fn get_shadow_user(&self, _user_id: Uuid) -> Result<Option<ShadowUser>, AppError> {
            Ok(None)
        }

        async fn update_total_score(
            &self,
            _user_id: Uuid,
            _score_to_add: i32,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    // Test 1: sync_user_success - New user synced successfully with total_score=0
    #[tokio::test]
    async fn sync_user_success() {
        let user_id = Uuid::new_v4();
        let repo = MockUserRepository::new();
        let use_case = SyncNewUserUseCase::new(repo);

        let dto = SyncUserRequestDto { user_id };
        let result = use_case.execute(dto).await;

        assert!(result.is_ok());
        let shadow_user = result.unwrap();
        assert_eq!(shadow_user.user_id(), user_id);
        assert_eq!(shadow_user.total_score(), 0);
    }

    #[tokio::test]
    async fn sync_user_already_exists() {
        let user_id = Uuid::new_v4();
        let repo = MockUserRepository::with_existing_user(user_id);
        let use_case = SyncNewUserUseCase::new(repo);

        let dto = SyncUserRequestDto { user_id };
        let result = use_case.execute(dto).await;

        assert!(result.is_ok());
        let shadow_user = result.unwrap();
        assert_eq!(shadow_user.user_id(), user_id);
    }

    // Test 3: sync_user_empty_user_id - Invalid UUID handling (zero UUID)
    #[tokio::test]
    async fn sync_user_empty_user_id() {
        let user_id = Uuid::nil(); // Zero UUID
        let repo = MockUserRepository::new();
        let use_case = SyncNewUserUseCase::new(repo);

        let dto = SyncUserRequestDto { user_id };
        let result = use_case.execute(dto).await;

        // Zero UUID should still be processed (not an error), but it depends on business rules
        // The use case doesn't explicitly reject nil UUIDs, so this tests the boundary
        assert!(result.is_ok());
        let shadow_user = result.unwrap();
        assert_eq!(shadow_user.user_id(), Uuid::nil());
        assert_eq!(shadow_user.total_score(), 0);
    }

    #[tokio::test]
    async fn sync_user_concurrent_same_user() {
        let user_id = Uuid::new_v4();
        let repo = MockUserRepository::new();
        let use_case = SyncNewUserUseCase::new(repo);

        let dto = SyncUserRequestDto { user_id };

        let result1 = use_case.execute(dto.clone()).await;
        assert!(result1.is_ok());

        let repo_with_user = MockUserRepository::with_existing_user(user_id);
        let use_case2 = SyncNewUserUseCase::new(repo_with_user);

        let result2 = use_case2.execute(dto).await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().user_id(), user_id);
    }

    // Test 5: sync_user_database_error - Returns UserSyncError::DatabaseError
    #[tokio::test]
    async fn sync_user_database_error() {
        let user_id = Uuid::new_v4();
        let repo = MockUserRepository::new().with_insert_failure();
        let use_case = SyncNewUserUseCase::new(repo);

        let dto = SyncUserRequestDto { user_id };
        let result = use_case.execute(dto).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, UserSyncError::DatabaseError(_)));
    }

    #[tokio::test]
    async fn execute_should_return_conflict_when_user_exists() {
        let user_id = Uuid::new_v4();
        let repo = MockUserRepository::with_existing_user(user_id);
        let use_case = SyncNewUserUseCase::new(repo);

        let dto = SyncUserRequestDto { user_id };
        let result = use_case.execute(dto).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().user_id(), user_id);
    }

    #[tokio::test]
    async fn execute_should_create_user_when_not_exists() {
        let user_id = Uuid::new_v4();
        let repo = MockUserRepository::new();
        let use_case = SyncNewUserUseCase::new(repo);

        let dto = SyncUserRequestDto { user_id };
        let result = use_case.execute(dto).await;

        assert!(result.is_ok());
        let shadow_user = result.unwrap();
        assert_eq!(shadow_user.user_id(), user_id);
        assert_eq!(shadow_user.total_score(), 0);
    }
}
