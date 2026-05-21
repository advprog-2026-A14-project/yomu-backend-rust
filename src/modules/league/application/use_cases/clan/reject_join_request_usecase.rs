use crate::modules::league::application::dto::join_request_dto::{ApproveRejectDto, JoinRequestResponseDto};
use crate::modules::league::domain::errors::LeagueError;
use crate::modules::league::domain::repositories::clan_join_request_repository::ClanJoinRequestRepository;
use crate::modules::league::domain::repositories::ClanRepository;
use tracing::instrument;
use uuid::Uuid;

pub struct RejectJoinRequestUseCase<R: ClanRepository, J: ClanJoinRequestRepository> {
    clan_repo: R,
    join_repo: J,
}

impl<R: ClanRepository, J: ClanJoinRequestRepository> RejectJoinRequestUseCase<R, J> {
    pub fn new(clan_repo: R, join_repo: J) -> Self {
        Self { clan_repo, join_repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(
        &self,
        request_id: Uuid,
        dto: ApproveRejectDto,
    ) -> Result<JoinRequestResponseDto, LeagueError> {
        // Get the join request
        let request = self
            .join_repo
            .get_request_by_id(request_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        let mut request = request
            .ok_or_else(|| LeagueError::RequestNotFound(request_id.to_string()))?;

        // Ensure request is still pending
        if !matches!(request.status(), crate::modules::league::domain::entities::clan_join_request::RequestStatus::Pending) {
            return Err(LeagueError::RequestAlreadyProcessed(
                format!("Request {} has already been {}", request_id, request.status())
            ));
        }

        // Validate caller is clan leader
        let clan = self
            .clan_repo
            .get_clan_by_id(request.clan_id())
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        let clan = clan
            .ok_or_else(|| LeagueError::ClanNotFound(request.clan_id().to_string()))?;
        if clan.leader_id() != dto.caller_id {
            return Err(LeagueError::NotLeader(
                "Only the clan leader can reject join requests".to_string(),
            ));
        }

        // Reject
        request.reject();
        self.join_repo
            .update_request_status(request.id(), request.status())
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
