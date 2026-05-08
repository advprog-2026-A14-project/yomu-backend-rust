//! Structured JSON logging with tracing-appender file rotation.
//!
//! Provides `init_logging()` which sets up:
//! - `EnvFilter` from RUST_LOG env var (default: "yomu_backend_rust=info,tower_http=warn")
//! - JSON file output via `fmt::layer().json()` to `tracing_appender::non_blocking`
//! - Hourly file rotation via `RollingFileAppender`
//!
//! LOG_DIR configurable via env var (default: "/var/log/yomu")

use super::telemetry::otel_layer;
use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Configuration for logging initialization.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Directory for log files.
    pub log_dir: String,
    /// Log level filter (RUST_LOG format).
    pub log_level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_dir: std::env::var("LOG_DIR").unwrap_or_else(|_| "/var/log/yomu".into()),
            log_level: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "yomu_backend_rust=info,tower_http=warn".into()),
        }
    }
}

/// Initialize structured logging with JSON file output and hourly rotation.
///
/// # Arguments
/// * `config` - Log configuration (uses defaults if None)
///
/// # Returns
/// * `WorkerGuard` - Must be kept alive until shutdown for graceful log flushing
///
/// # Example
/// ```
/// use yomu_backend_rust::shared::infrastructure::logging::{init_logging, LogConfig};
/// let config = LogConfig {
///     log_dir: std::env::var("LOG_DIR").unwrap_or_else(|_| "/tmp/yomu_test_logs".into()),
///     log_level: "info".into(),
/// };
/// let guard = init_logging(Some(config));
/// ```
pub fn init_logging(config: Option<LogConfig>) -> WorkerGuard {
    let config = config.unwrap_or_default();
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    // Try to create log directory, fall back to temp dir if permission denied
    let log_dir = match std::fs::create_dir_all(&config.log_dir) {
        Ok(_) => config.log_dir.clone(),
        Err(e) => {
            eprintln!(
                "Warning: Could not create log directory {}: {}, using /tmp/yomu_logs",
                config.log_dir, e
            );
            let fallback = "/tmp/yomu_logs".to_string();
            let _ = std::fs::create_dir_all(&fallback);
            fallback
        }
    };

    // File appender with hourly rotation
    let file_appender = RollingFileAppender::new(Rotation::HOURLY, Path::new(&log_dir), "yomu.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // JSON file output with trace context
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(otel_layer())
        .with(env_filter)
        .with(file_layer)
        .init();

    guard
}
