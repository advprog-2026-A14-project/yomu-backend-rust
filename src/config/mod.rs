pub mod database;
use std::env;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub grpc_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub java_core_url: String,
    pub java_core_api_key: String,
    pub jwt_secret: String,
    pub pg_pool_max_connections: u32,
    pub pg_pool_min_connections: u32,
    pub pg_pool_acquire_timeout_secs: u64,
    pub pg_pool_max_lifetime_secs: u64,
    pub pg_pool_idle_timeout_secs: u64,
}

impl AppConfig {
    #[allow(clippy::panic)]
    pub fn load() -> Self {
        let _ = dotenvy::dotenv();

        Self {
            host: get_env("APP_HOST", "0.0.0.0"),
            port: get_env("APP_PORT", "8080")
                .parse()
                .unwrap_or_else(|_| panic!("APP_PORT must be a number")),
            grpc_port: get_env_parse("GRPC_PORT", "9090"),
            database_url: get_env_strict("DATABASE_URL"),
            redis_url: get_env_strict("REDIS_URL"),
            java_core_url: get_env_strict("JAVA_CORE_URL"),
            java_core_api_key: get_env_strict("JAVA_CORE_API_KEY"),
            jwt_secret: get_env_strict("JWT_SECRET"),
            pg_pool_max_connections: get_env_parse("PG_POOL_MAX_CONNECTIONS", "50"),
            pg_pool_min_connections: get_env_parse("PG_POOL_MIN_CONNECTIONS", "5"),
            pg_pool_acquire_timeout_secs: get_env_parse("PG_POOL_ACQUIRE_TIMEOUT_SECS", "30"),
            pg_pool_max_lifetime_secs: get_env_parse("PG_POOL_MAX_LIFETIME_SECS", "1800"),
            pg_pool_idle_timeout_secs: get_env_parse("PG_POOL_IDLE_TIMEOUT_SECS", "300"),
        }
    }
}

fn get_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

#[allow(clippy::panic)]
fn get_env_strict(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Missing required environment variable: {}", key))
}

#[allow(clippy::panic)]
fn get_env_parse<T: std::str::FromStr>(key: &str, default: &str) -> T {
    env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .parse()
        .unwrap_or_else(|_| panic!("Environment variable {} must be a valid number", key))
}

