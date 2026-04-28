use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource, propagation::TraceContextPropagator, trace::Sampler, trace::SdkTracerProvider,
};

pub fn init_telemetry() -> Result<SdkTracerProvider, Box<dyn std::error::Error + Send + Sync>> {
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "yomu-backend-rust".into());
    let environment = std::env::var("OTEL_ENVIRONMENT").unwrap_or_else(|_| "development".into());
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".into());
    let sampling_ratio: f64 = std::env::var("OTEL_TRACES_SAMPLER_ARG")
        .unwrap_or_else(|_| "0.1".into())
        .parse()
        .unwrap_or(0.1);

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .build()?;

    let resource = Resource::builder()
        .with_attributes([
            opentelemetry::KeyValue::new("service.name", service_name),
            opentelemetry::KeyValue::new("deployment.environment", environment),
            opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])
        .build();

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            sampling_ratio,
        ))))
        .with_batch_exporter(exporter)
        .build();

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    Ok(tracer_provider)
}

use tracing_subscriber::layer::Layer;

pub fn otel_layer() -> impl Layer<tracing_subscriber::Registry> {
    tracing_opentelemetry::layer()
}
