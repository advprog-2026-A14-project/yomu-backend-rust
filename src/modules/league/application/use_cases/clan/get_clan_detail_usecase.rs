use crate::modules::league::application::dto::{
    BuffInfo, ClanDetailDto, ClanMemberDto, DebuffInfo,
};
use crate::modules::league::domain::errors::LeagueError;
use crate::modules::league::domain::repositories::ClanBuffRepository;
use crate::modules::league::domain::repositories::ClanRepository;
use uuid::Uuid;

pub struct GetClanDetailUseCase<R: ClanRepository, B: ClanBuffRepository> {
    repository: R,
    buff_repository: B,
}

impl<R: ClanRepository, B: ClanBuffRepository> GetClanDetailUseCase<R, B> {
    pub fn new(repository: R, buff_repository: B) -> Self {
        Self {
            repository,
            buff_repository,
        }
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

        let buffs = self
            .buff_repository
            .get_active_buffs(clan_id)
            .await
            .map_err(|e| LeagueError::ClanNotFound(e.to_string()))?;

        let mut active_buffs: Vec<BuffInfo> = Vec::new();
        let mut active_debuffs: Vec<DebuffInfo> = Vec::new();

        for buff in buffs {
            if buff.is_debuff() {
                active_debuffs.push(DebuffInfo {
                    name: buff.buff_name().to_string(),
                    multiplier: buff.multiplier(),
                    expires_at: buff.expires_at(),
                });
            } else {
                active_buffs.push(BuffInfo {
                    name: buff.buff_name().to_string(),
                    multiplier: buff.multiplier(),
                    expires_at: buff.expires_at(),
                });
            }
        }

        Ok(ClanDetailDto {
            id: clan.id(),
            name: clan.name().to_string(),
            leader_id: clan.leader_id(),
            tier: clan.tier().to_string(),
            total_score: clan.total_score(),
            created_at: clan.created_at(),
            members: member_dtos,
            active_buffs,
            active_debuffs,
        })
    }
}
