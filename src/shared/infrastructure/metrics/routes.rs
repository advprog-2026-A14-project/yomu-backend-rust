use axum::{Extension, Router, routing::get};
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;

pub fn metrics_routes() -> Router {
    Router::new().route("/metrics", get(metrics_handler))
}

async fn metrics_handler(
    Extension(handle): Extension<PrometheusHandle>,
) -> impl axum::response::IntoResponse {
    let buffer = handle.render();
    (
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        buffer,
    )
}
