use redis::{Client, aio::MultiplexedConnection};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use tracing::info;

use super::AppConfig;

pub async fn init_postgres_pool(config: &AppConfig) -> Result<PgPool, sqlx::Error> {
    info!("Trying to connect to PostgreSQL...");

    let pool = PgPoolOptions::new()
        .max_connections(config.pg_pool_max_connections)
        .min_connections(config.pg_pool_min_connections)
        .acquire_timeout(Duration::from_secs(config.pg_pool_acquire_timeout_secs))
        .max_lifetime(Duration::from_secs(config.pg_pool_max_lifetime_secs))
        .idle_timeout(Duration::from_secs(config.pg_pool_idle_timeout_secs))
        .test_before_acquire(true)
        .connect(&config.database_url)
        .await?;

    info!("Connected to PostgreSQL successfully!");

    Ok(pool)
}

pub async fn init_redis_pool(redis_url: &str) -> Result<MultiplexedConnection, redis::RedisError> {
    info!("Trying to connect to Redis...");

    let client = Client::open(redis_url)?;

    let con = client.get_multiplexed_async_connection().await?;

    info!("Connected to Redis successfully!");

    Ok(con)
}
