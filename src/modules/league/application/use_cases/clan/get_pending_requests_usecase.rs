use crate::modules::league::application::dto::join_request_dto::JoinRequestResponseDto;
use crate::modules::league::domain::errors::LeagueError;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::modules::league::domain::repositories::clan_join_request_repository::ClanJoinRequestRepository;
use tracing::instrument;
use uuid::Uuid;

pub struct GetPendingRequestsUseCase<R: ClanRepository, J: ClanJoinRequestRepository> {
    clan_repo: R,
    join_repo: J,
}

impl<R: ClanRepository, J: ClanJoinRequestRepository> GetPendingRequestsUseCase<R, J> {
    pub fn new(clan_repo: R, join_repo: J) -> Self {
        Self {
            clan_repo,
            join_repo,
        }
    }

    #[instrument(skip(self))]
    pub async fn execute(
        &self,
        clan_id: Uuid,
        caller_id: Uuid,
    ) -> Result<Vec<JoinRequestResponseDto>, LeagueError> {
        tracing::info!(%clan_id, "Executing get pending requests");
        // Validate clan exists
        let clan = self
            .clan_repo
            .get_clan_by_id(clan_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        let clan = clan.ok_or_else(|| LeagueError::ClanNotFound(clan_id.to_string()))?;

        // Only leader can view pending requests
        if clan.leader_id() != caller_id {
            return Err(LeagueError::NotLeader(
                "Only the clan leader can view pending requests".to_string(),
            ));
        }

        let requests = self
            .join_repo
            .get_pending_requests_by_clan(clan_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;

        let dtos = requests
            .into_iter()
            .map(|r| JoinRequestResponseDto {
                id: r.id(),
                clan_id: r.clan_id(),
                user_id: r.user_id(),
                status: r.status().to_string(),
                created_at: r.created_at(),
                updated_at: r.updated_at(),
            })
            .collect();

        Ok(dtos)
    }
}
