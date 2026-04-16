use crate::modules::league::application::dto::{ClanDetailDto, ClanMemberDto};
use crate::modules::league::domain::entities::clan::Clan;
use crate::modules::league::domain::entities::clan_member::ClanMember;
use crate::modules::league::domain::errors::LeagueError;
use crate::modules::league::domain::repositories::ClanRepository;
use uuid::Uuid;

pub struct GetClanDetailUseCase<R: ClanRepository> {
    repository: R,
}

impl<R: ClanRepository> GetClanDetailUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, clan_id: Uuid) -> Result<ClanDetailDto, LeagueError> {
        let clan = self
            .repository
            .get_clan_by_id(clan_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;

        let clan = clan.ok_or_else(|| LeagueError::ClanNotFound(clan_id.to_string()))?;

        let members = self
            .repository
            .get_members_by_clan_id(clan_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;

        let member_dtos: Vec<ClanMemberDto> = members
            .iter()
            .map(|m| ClanMemberDto {
                user_id: m.user_id(),
                role: m.role().to_string(),
                joined_at: m.joined_at(),
            })
            .collect();

        Ok(ClanDetailDto {
            id: clan.id(),
            name: clan.name().to_string(),
            leader_id: clan.leader_id(),
            tier: clan.tier().to_string(),
            total_score: clan.total_score(),
            created_at: clan.created_at(),
            members: member_dtos,
            active_buffs: vec![],
            active_debuffs: vec![],
        })
    }
}
