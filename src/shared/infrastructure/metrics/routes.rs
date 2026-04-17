use axum::{Extension, Router, routing::get};
use prometheus_client::encoding::text::encode;

pub fn metrics_routes() -> Router<crate::AppState> {
    Router::new().route("/metrics", get(metrics_handler))
}

async fn metrics_handler(
    Extension(metrics): Extension<
        std::sync::Arc<crate::shared::infrastructure::metrics::AppMetrics>,
    >,
) -> impl axum::response::IntoResponse {
    let mut buffer = String::new();
    #[allow(clippy::expect_used)]
    encode(&mut buffer, &metrics.registry).expect("metrics encoding should not fail");
    (
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        buffer,
    )
}
