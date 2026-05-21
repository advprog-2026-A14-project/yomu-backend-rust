use crate::modules::league::application::dto::join_request_dto::{CreateJoinRequestDto, JoinRequestResponseDto};
use crate::modules::league::domain::entities::clan_join_request::ClanJoinRequest;
use crate::modules::league::domain::errors::LeagueError;
use crate::modules::league::domain::repositories::clan_join_request_repository::ClanJoinRequestRepository;
use crate::modules::league::domain::repositories::ClanRepository;
use tracing::instrument;

pub struct CreateJoinRequestUseCase<R: ClanRepository, J: ClanJoinRequestRepository> {
    clan_repo: R,
    join_repo: J,
}

impl<R: ClanRepository, J: ClanJoinRequestRepository> CreateJoinRequestUseCase<R, J> {
    pub fn new(clan_repo: R, join_repo: J) -> Self {
        Self { clan_repo, join_repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, dto: CreateJoinRequestDto) -> Result<JoinRequestResponseDto, LeagueError> {
        // Validate clan exists
        let clan = self
            .clan_repo
            .get_clan_by_id(dto.clan_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        let _clan = clan
            .ok_or_else(|| LeagueError::ClanNotFound(dto.clan_id.to_string()))?;

        // User must not be in any clan
        let in_clan = self
            .clan_repo
            .is_user_in_any_clan(dto.user_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        if in_clan {
            return Err(LeagueError::UserAlreadyInClan(
                "User is already in a clan".to_string(),
            ));
        }

        // Check no pending request exists for this user+clan
        let existing = self
            .join_repo
            .get_pending_request_by_user(dto.user_id, dto.clan_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        if existing.is_some() {
            return Err(LeagueError::DuplicateRequest(
                "User already has a pending join request for this clan".to_string(),
            ));
        }

        // Create the pending request
        let request = ClanJoinRequest::new(dto.clan_id, dto.user_id);
        self.join_repo
            .create_request(&request)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;

        Ok(JoinRequestResponseDto {
            id: request.id(),
            clan_id: request.clan_id(),
            user_id: request.user_id(),
            status: request.status().to_string(),
            created_at: request.created_at(),
            updated_at: request.updated_at(),
        })
    }
}
