use crate::modules::league::application::dto::join_request_dto::{
    ApproveRejectDto, JoinRequestResponseDto,
};
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::modules::league::domain::entities::clan_member::MemberRole;
use crate::modules::league::domain::errors::LeagueError;
use crate::modules::league::domain::repositories::ClanRepository;
use crate::modules::league::domain::repositories::clan_join_request_repository::ClanJoinRequestRepository;
use tracing::instrument;
use uuid::Uuid;

pub struct ApproveJoinRequestUseCase<R: ClanRepository, J: ClanJoinRequestRepository> {
    clan_repo: R,
    join_repo: J,
}

impl<R: ClanRepository, J: ClanJoinRequestRepository> ApproveJoinRequestUseCase<R, J> {
    pub fn new(clan_repo: R, join_repo: J) -> Self {
        Self {
            clan_repo,
            join_repo,
        }
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
        let mut request =
            request.ok_or_else(|| LeagueError::RequestNotFound(request_id.to_string()))?;

        // Ensure request is still pending
        if !matches!(
            request.status(),
            crate::modules::league::domain::entities::clan_join_request::RequestStatus::Pending
        ) {
            return Err(LeagueError::RequestAlreadyProcessed(format!(
                "Request {} has already been {}",
                request_id,
                request.status()
            )));
        }

        // Validate caller is clan leader
        let clan = self
            .clan_repo
            .get_clan_by_id(request.clan_id())
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        let clan = clan.ok_or_else(|| LeagueError::ClanNotFound(request.clan_id().to_string()))?;
        if clan.leader_id() != dto.caller_id {
            return Err(LeagueError::NotLeader(
                "Only the clan leader can approve join requests".to_string(),
            ));
        }

        // Verify user is still not in any clan
        let in_clan = self
            .clan_repo
            .is_user_in_any_clan(request.user_id())
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
        if in_clan {
            // Auto-reject if user already joined another clan
            request.reject();
            self.join_repo
                .update_request_status(request.id(), request.status())
                .await
                .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;
            return Err(LeagueError::UserAlreadyInClan(
                "User is already in another clan".to_string(),
            ));
        }

        // Approve and add as member
        request.approve();
        self.join_repo
            .update_request_status(request.id(), request.status())
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;

        let member = ClanMember::new(request.clan_id(), request.user_id(), MemberRole::Member);
        self.clan_repo
            .add_member(&member)
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
