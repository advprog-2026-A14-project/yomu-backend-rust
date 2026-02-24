mod config;

use axum::{Router, http::StatusCode, response::Json, routing::get};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};
use sqlx::PgPool; 
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

}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health_check(State(_state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }),
    )
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
            tracing::error!("Gagal terhubung ke database: {}", e);
            std::process::exit(1);
        }
    };

    let state = AppState {
        db: db_pool,
    };

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

        let app = Router::new()
        .route("/health", get(health_check))
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
