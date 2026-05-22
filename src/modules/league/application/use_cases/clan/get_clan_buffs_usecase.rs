use crate::modules::league::application::dto::{BuffDto, ClanBuffsDto, DebuffDto};
use crate::modules::league::domain::repositories::ClanBuffRepository;
use crate::shared::domain::base_error::AppError;
use tracing::instrument;
use uuid::Uuid;

pub struct GetClanBuffsUseCase<R: ClanBuffRepository> {
    repo: R,
}

impl<R: ClanBuffRepository> GetClanBuffsUseCase<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn execute(&self, clan_id: Uuid) -> Result<ClanBuffsDto, AppError> {
        tracing::info!(%clan_id, "Executing get clan buffs");
        let active_buffs = self.repo.get_active_buffs(clan_id).await?;

        let mut buffs: Vec<BuffDto> = Vec::new();
        let mut debuffs: Vec<DebuffDto> = Vec::new();

        for buff in active_buffs {
            let expires_at = buff.expires_at().to_rfc3339();
            if buff.is_debuff() {
                debuffs.push(DebuffDto {
                    name: buff.buff_name().to_string(),
                    multiplier: buff.multiplier(),
                    expires_at,
                });
            } else {
                buffs.push(BuffDto {
                    name: buff.buff_name().to_string(),
                    multiplier: buff.multiplier(),
                    expires_at,
                });
            }
        }

        Ok(ClanBuffsDto { buffs, debuffs })
    }
}
