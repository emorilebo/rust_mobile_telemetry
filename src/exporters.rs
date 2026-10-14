//! Exporter pipelines for the two supported back ends: stdout (`Console`) and
//! OTLP over gRPC (`Otlp`).
//!
//! Most applications only need [`init_telemetry`](crate::init_telemetry); these
//! functions are public for callers who want to assemble the pieces themselves.
//!
//! Background export work (batching, periodic metric collection, the gRPC
//! client) runs on a small Tokio runtime owned by this crate, so the pipelines
//! work from synchronous code and from any Tokio runtime flavour, and
//! [`shutdown_telemetry`](crate::shutdown_telemetry) can block on the final
//! flush without deadlocking the caller's runtime.

use std::sync::OnceLock;
use std::time::Duration;

use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::{MeterProvider, PeriodicReader};
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::{self, Tracer, TracerProvider};
use opentelemetry_sdk::Resource;
use tokio::runtime::Runtime;

use crate::config::Config;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Endpoint used by the OTLP exporter when none is configured.
pub const DEFAULT_OTLP_ENDPOINT: &str = "http://localhost:4317";

/// How often the OTLP metrics reader pushes data.
const OTLP_METRICS_PERIOD: Duration = Duration::from_secs(3);

static EXPORT_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Returns the crate-owned runtime that runs background export tasks, creating
/// it on first use.
pub(crate) fn export_runtime() -> std::io::Result<&'static Runtime> {
    if let Some(runtime) = EXPORT_RUNTIME.get() {
        return Ok(runtime);
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("rust-mobile-telemetry")
        .enable_all()
        .build()?;
    // Lost a race with another thread: discard ours without blocking (dropping a
    // runtime inside an async context panics).
    if let Err(runtime) = EXPORT_RUNTIME.set(runtime) {
        runtime.shutdown_background();
    }
    Ok(EXPORT_RUNTIME
        .get()
        .expect("export runtime is initialised above"))
}

/// Builds a tracer that prints finished spans as JSON lines to stdout, and
/// registers its provider as the global OpenTelemetry tracer provider.
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

    global::set_tracer_provider(provider.clone());
    provider.tracer("rust_mobile_telemetry")
}

/// Builds a meter provider that prints metrics as JSON lines to stdout every
/// 60 seconds and on [`shutdown_telemetry`](crate::shutdown_telemetry).
///
/// # Panics
///
/// Panics if the background export runtime cannot be created (the OS refused
/// to spawn its thread). [`init_telemetry`](crate::init_telemetry) reports that
/// case as an error instead.
pub fn init_console_meter_provider(resource: Resource) -> MeterProvider {
    let runtime = export_runtime().expect("rust_mobile_telemetry: failed to start export runtime");
    let _context = runtime.enter();

    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();

    MeterProvider::builder()
        .with_resource(resource)
        .with_reader(reader)
        .build()
}

/// Builds a batching OTLP/gRPC span exporter and registers its provider as the
/// global OpenTelemetry tracer provider.
///
/// The endpoint is `config.otlp_endpoint`, falling back to
/// [`DEFAULT_OTLP_ENDPOINT`]. Note that `opentelemetry-otlp` lets the
/// `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` / `OTEL_EXPORTER_OTLP_ENDPOINT`
/// environment variables override it.
///
/// # Errors
///
/// Fails if the endpoint is not a valid URI or the export runtime cannot start.
pub fn init_otlp_tracer(config: &Config, resource: Resource) -> Result<Tracer, BoxError> {
    let endpoint = config
        .otlp_endpoint
        .as_deref()
        .unwrap_or(DEFAULT_OTLP_ENDPOINT);
    let _context = export_runtime()?.enter();

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
                .with_sampler(trace::Sampler::AlwaysOn),
        )
        .install_batch(runtime::Tokio)
        .map_err(|e| e.into())
}

/// Builds an OTLP/gRPC metrics pipeline that pushes every 3 seconds and on
/// [`shutdown_telemetry`](crate::shutdown_telemetry).
///
/// Endpoint resolution is the same as for [`init_otlp_tracer`]
/// (`OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` / `OTEL_EXPORTER_OTLP_ENDPOINT` win).
///
/// # Errors
///
/// Fails if the endpoint is not a valid URI or the export runtime cannot start.
pub fn init_otlp_meter_provider(
    config: &Config,
    resource: Resource,
) -> Result<MeterProvider, BoxError> {
    let endpoint = config
        .otlp_endpoint
        .as_deref()
        .unwrap_or(DEFAULT_OTLP_ENDPOINT);
    let _context = export_runtime()?.enter();

    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .with_resource(resource)
        .with_period(OTLP_METRICS_PERIOD)
        .build()
        .map_err(|e| e.into())
}
