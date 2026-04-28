mod config;
mod modules;
mod shared;

use crate::shared::domain::base_error::AppError;
use crate::shared::infrastructure::logging::init_logging;
use crate::shared::infrastructure::metrics::routes::metrics_routes;
use crate::shared::infrastructure::telemetry::{init_telemetry, init_telemetry_subscriber};
use crate::shared::utils::response::ApiResponse;
use axum::{Extension, Router, extract::State, response::Json, routing::get};
use axum_prometheus::PrometheusMetricLayer;
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use yomu_backend_rust::{ApiDoc, AppMetrics, AppState, HealthResponse};

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy")
    ),
    tag = "health"
)]
async fn health_check(
    State(state): State<AppState>,
) -> (axum::http::StatusCode, Json<ApiResponse<HealthResponse>>) {
    let postgres_status = match sqlx::query("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => "connected".to_string(),
        Err(e) => format!("error: {}", e),
    };

    let mut redis_conn = state.redis.clone();
    let redis_status = match redis::cmd("PING")
        .query_async::<String>(&mut redis_conn)
        .await
    {
        Ok(resp) if resp == "PONG" => "connected".to_string(),
        Ok(resp) => resp,
        Err(e) => format!("error: {}", e),
    };

    let overall_status = if postgres_status == "connected" && redis_status == "connected" {
        "healthy"
    } else {
        "unhealthy"
    };

    let health_data = HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        postgres: postgres_status,
        redis: redis_status,
    };

    let response = ApiResponse::success("Server is running well", health_data);

    let status_code = if overall_status == "healthy" {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(response))
}

async fn simulate_error() -> Result<Json<ApiResponse<()>>, AppError> {
    Err(AppError::NotFound(
        "Clan atau User tidak ditemukan di database".to_string(),
    ))
}

fn main() {
    let _log_guard = init_logging(None);

    tracing::info!("Starting Yomu Engine Rust...");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async_main_internal());
}

async fn async_main_internal() {
    let _tracer_provider = init_telemetry().expect("Failed to initialize telemetry");
    init_telemetry_subscriber();
    async_main(config::AppConfig::load()).await;
}

async fn async_main(app_config: config::AppConfig) {
    let db_pool = match config::database::init_postgres_pool(&app_config.database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed connecting to database: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Running database migrations...");
    if let Err(e) = sqlx::migrate!().run(&db_pool).await {
        tracing::error!("Error while doing database migrations: {}", e);
        std::process::exit(1);
    }
    tracing::info!("Database migration complete!");

    let redis_pool = match config::database::init_redis_pool(&app_config.redis_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed connecting to Redis: {}", e);
            std::process::exit(1)
        }
    };

    let metrics = Arc::new(AppMetrics::new());
    let (prometheus_layer, _) = PrometheusMetricLayer::pair();

    let state = AppState {
        db: db_pool,
        redis: redis_pool,
        metrics: metrics.clone(),
    };

    let middleware_stack = ServiceBuilder::new()
        .layer(CompressionLayer::new())
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let api_v1_router = Router::new().merge(modules::league::presentation::routes::league_routes());
    let internal_api_router =
        Router::new().merge(modules::user_sync::presentation::routes::user_sync_routes());

    let swagger = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi());

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/error", get(simulate_error))
        .merge(swagger)
        .merge(metrics_routes())
        .nest("/api/v1", api_v1_router)
        .nest("/api/internal", internal_api_router)
        .with_state(state)
        .layer(middleware_stack)
        .layer(Extension(metrics))
        .layer(OtelAxumLayer::default());

    #[allow(clippy::expect_used)]
    let addr: SocketAddr = format!("{}:{}", app_config.host, app_config.port)
        .parse()
        .expect("Invalid host/port configuration");

    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to bind TCP listener on {}: {}", addr, e);
            std::process::exit(1);
        });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Axum server error: {}", e);
            std::process::exit(1);
        });
}

#[allow(clippy::expect_used)]
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
