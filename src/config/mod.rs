use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub java_core_url: String,
    pub java_core_api_key: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let _ = dotenvy::dotenv();

        Self {
            host: get_env("APP_HOST", "0.0.0.0"),
            port: get_env("APP_PORT", "8080").parse().expect("APP_PORT must be a number"),
            database_url: get_env_strict("DATABASE_URL"),
            redis_url: get_env_strict("REDIS_URL"),
            java_core_url: get_env_strict("JAVA_CORE_URL"),
            java_core_api_key: get_env_strict("JAVA_CORE_API_KEY"),
        }
    }
}

fn get_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn get_env_strict(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("Missing required environment variable: {}", key))
}