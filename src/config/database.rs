use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

use tracing::{error, info};
pub async fn init_postgres_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Trying to connect to PostgreSQL...");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .test_before_acquire(true)
        .connect(database_url)
        .await?;

    info!("Connected to PostgreSQL successfully!");
    
    Ok(pool)
}
