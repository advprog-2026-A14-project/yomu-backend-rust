use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use prometheus_client::metrics::{
    counter::Counter, family::Family, gauge::Gauge, histogram::Histogram,
};
use prometheus_client::registry::Registry;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HttpLabels {
    pub method: String,
    pub path: String,
    pub status: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl From<&axum::http::Method> for HttpMethod {
    fn from(method: &axum::http::Method) -> Self {
        match *method {
            axum::http::Method::GET => HttpMethod::GET,
            axum::http::Method::POST => HttpMethod::POST,
            axum::http::Method::PUT => HttpMethod::PUT,
            axum::http::Method::DELETE => HttpMethod::DELETE,
            axum::http::Method::PATCH => HttpMethod::PATCH,
            _ => HttpMethod::GET,
        }
    }
}

pub struct AppMetrics {
    pub registry: Registry,
    pub http_requests: Family<HttpLabels, Counter>,
    pub http_request_duration: Family<HttpLabels, Histogram, fn() -> Histogram>,
    pub db_pool_idle: Gauge,
    pub db_pool_active: Gauge,
    pub redis_pool_idle: Gauge,
    pub redis_pool_active: Gauge,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
}

impl AppMetrics {
    pub fn new() -> Self {
        let mut registry = Registry::default();

        let http_requests: Family<HttpLabels, Counter> = Family::default();
        registry.register(
            "http_requests_total",
            "Total number of HTTP requests",
            http_requests.clone(),
        );

        let http_request_duration = Family::new_with_constructor(|| {
            Histogram::new(
                [
                    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ]
                .into_iter(),
            )
        });
        registry.register(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
            http_request_duration.clone(),
        );

        let db_pool_idle = Gauge::default();
        let db_pool_active = Gauge::default();
        registry.register(
            "db_pool_idle_connections",
            "Idle DB connections",
            db_pool_idle.clone(),
        );
        registry.register(
            "db_pool_active_connections",
            "Active DB connections",
            db_pool_active.clone(),
        );

        let redis_pool_idle = Gauge::default();
        let redis_pool_active = Gauge::default();
        registry.register(
            "redis_pool_idle_connections",
            "Idle Redis connections",
            redis_pool_idle.clone(),
        );
        registry.register(
            "redis_pool_active_connections",
            "Active Redis connections",
            redis_pool_active.clone(),
        );

        let cache_hits = Counter::default();
        let cache_misses = Counter::default();
        registry.register("cache_hits_total", "Cache hits", cache_hits.clone());
        registry.register("cache_misses_total", "Cache misses", cache_misses.clone());

        Self {
            registry,
            http_requests,
            http_request_duration,
            db_pool_idle,
            db_pool_active,
            redis_pool_idle,
            redis_pool_active,
            cache_hits,
            cache_misses,
        }
    }

    pub fn encode(&self) -> String {
        use prometheus_client::encoding::text::encode;
        let mut buffer = String::new();
        encode(&mut buffer, &self.registry).unwrap();
        buffer
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new()
    }
}
