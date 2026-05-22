use tonic::{Request, Response, Status};

use crate::generated::league::{
    GetLeaderboardRequest, GetLeaderboardResponse, GetUserTierRequest, GetUserTierResponse,
    JoinClanRequest, JoinClanResponse, LeaderboardEntry, league_service_server::LeagueService,
};
use crate::modules::league::application::use_cases::GetLeaderboardUseCase;
use crate::modules::league::infrastructure::database::postgres::ClanPostgresRepo;
use crate::modules::league::infrastructure::database::redis::LeaderboardRedisRepo;
use redis::aio::MultiplexedConnection;

pub struct LeagueGrpcHandler {
    redis: MultiplexedConnection,
    db: sqlx::PgPool,
}

impl LeagueGrpcHandler {
    pub fn new(redis: MultiplexedConnection, db: sqlx::PgPool) -> Self {
        Self { redis, db }
    }
}

#[tonic::async_trait]
impl LeagueService for LeagueGrpcHandler {
    async fn get_user_tier(
        &self,
        request: Request<GetUserTierRequest>,
    ) -> Result<Response<GetUserTierResponse>, Status> {
        let req = request.into_inner();
        let _user_id = uuid::Uuid::parse_str(&req.user_id)
            .map_err(|_| Status::invalid_argument("invalid user_id"))?;

        Ok(Response::new(GetUserTierResponse {
            user_id: req.user_id,
            clan_id: "".to_string(),
            clan_name: "".to_string(),
            tier: "".to_string(),
        }))
    }

    async fn get_leaderboard(
        &self,
        request: Request<GetLeaderboardRequest>,
    ) -> Result<Response<GetLeaderboardResponse>, Status> {
        let req = request.into_inner();
        let redis_repo = LeaderboardRedisRepo::new(self.redis.clone());
        let clan_repo = ClanPostgresRepo::new(self.db.clone());
        let use_case = GetLeaderboardUseCase::new(clan_repo, redis_repo);

        let result = use_case.execute(req.tier.clone()).await;

        match result {
            Ok(leaderboard) => {
                let entries = leaderboard
                    .entries
                    .into_iter()
                    .map(|entry| LeaderboardEntry {
                        clan_id: entry.clan_id.to_string(),
                        clan_name: entry.clan_name,
                        total_score: entry.total_score,
                        tier: leaderboard.tier.clone(),
                    })
                    .collect();
                Ok(Response::new(GetLeaderboardResponse { entries }))
            }
            Err(_) => Err(Status::internal("leaderboard error")),
        }
    }

    async fn join_clan(
        &self,
        _request: Request<JoinClanRequest>,
    ) -> Result<Response<JoinClanResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
}
