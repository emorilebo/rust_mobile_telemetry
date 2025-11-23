use opentelemetry_sdk::trace::{self, Tracer, TracerProvider};
use opentelemetry_sdk::metrics::{MeterProvider, PeriodicReader};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::runtime;
use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use std::time::Duration;

use crate::config::Config;

pub fn init_console_tracer(resource: Resource) -> Tracer {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_config(
            trace::config()
                .with_resource(resource)
                .with_sampler(trace::Sampler::AlwaysOn),
        )
        .build();
    
    // Set global provider if needed, or just return the tracer
    global::set_tracer_provider(provider.clone());
    provider.tracer("rust_mobile_telemetry")
}

pub fn init_console_meter_provider(resource: Resource) -> MeterProvider {
    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
    
    MeterProvider::builder()
        .with_resource(resource)
        .with_reader(reader)
        .build()
}

pub fn init_otlp_tracer(config: &Config, resource: Resource) -> Result<Tracer, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let endpoint = config.otlp_endpoint.as_deref().unwrap_or("http://localhost:4317");
    
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .with_trace_config(
            trace::config()
                .with_resource(resource)
                .with_sampler(trace::Sampler::AlwaysOn), // TODO: Make configurable
        )
        .install_batch(runtime::Tokio)
        .map_err(|e| e.into())
}

pub fn init_otlp_meter_provider(config: &Config, resource: Resource) -> Result<MeterProvider, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let endpoint = config.otlp_endpoint.as_deref().unwrap_or("http://localhost:4317");

    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .with_resource(resource)
        .with_period(Duration::from_secs(3)) // Default export period
        .build()
        .map_err(|e| e.into())
}
