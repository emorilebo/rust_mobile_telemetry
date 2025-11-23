use opentelemetry::sdk::trace::{self, Tracer};
use opentelemetry::sdk::metrics::MeterProvider;
use opentelemetry::sdk::Resource;
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use std::time::Duration;

use crate::config::Config;

pub fn init_console_tracer(resource: Resource) -> Tracer {
    opentelemetry_stdout::new_pipeline()
        .trace()
        .with_trace_config(
            trace::config()
                .with_resource(resource)
                .with_sampler(trace::Sampler::AlwaysOn),
        )
        .install_simple()
}

pub fn init_console_meter_provider(resource: Resource) -> MeterProvider {
    opentelemetry_stdout::new_pipeline()
        .metrics()
        .with_resource(resource)
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
        .install_batch(opentelemetry::runtime::Tokio)
        .map_err(|e| e.into())
}

pub fn init_otlp_meter_provider(config: &Config, resource: Resource) -> Result<MeterProvider, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let endpoint = config.otlp_endpoint.as_deref().unwrap_or("http://localhost:4317");

    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry::runtime::Tokio)
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
