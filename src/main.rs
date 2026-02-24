mod config;
mod modules;
mod shared;

use crate::shared::domain::base_error::AppError;
use crate::shared::utils::response::ApiResponse;
use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{net::SocketAddr, time::Duration};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: MultiplexedConnection,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health_check(
    State(_state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<HealthResponse>>) {
    let health_data = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let response = ApiResponse::success("Server is running well", health_data);

    (StatusCode::OK, Json(response))
}

async fn simulate_error() -> Result<Json<ApiResponse<()>>, AppError> {
    Err(AppError::NotFound(
        "Clan atau User tidak ditemukan di database".to_string(),
    ))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "yomu_backend_rust=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Yomu Engine Rust...");

    let app_config = config::AppConfig::load();

    let db_pool = match config::database::init_postgres_pool(&app_config.database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed connecting to database: {}", e);
            std::process::exit(1);
        }
    };

    let redis_pool = match config::database::init_redis_pool(&app_config.redis_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed connecting to Redis: {}", e);
            std::process::exit(1)
        }
    };

    let state = AppState {
        db: db_pool,
        redis: redis_pool,
    };

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/error", get(simulate_error))
        .nest(
            "/api/users",
            modules::user_sync::presentation::routes::user_sync_routes(),
        )
        .with_state(state)
        .layer(middleware_stack);

    let addr: SocketAddr = format!("{}:{}", app_config.host, app_config.port)
        .parse()
        .expect("Invalid host/port configuration");

    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown...");
}
