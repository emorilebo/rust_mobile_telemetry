//! Small helpers for creating spans and metric instruments.
//!
//! Spans go through the `tracing` subscriber installed by
//! [`init_telemetry`](crate::init_telemetry); metric instruments come from the
//! global OpenTelemetry meter provider it registers. Both are no-ops (but safe
//! to call) if telemetry was never initialised.

use opentelemetry::global;
use opentelemetry::metrics::Counter;
use opentelemetry::metrics::Histogram;
use std::future::Future;
use std::time::Instant;
use tracing::Level;

/// Name of the OpenTelemetry instrumentation scope used for spans and metrics.
const INSTRUMENTATION_SCOPE: &str = "rust_mobile_telemetry";

/// Runs `f` inside an `INFO` span called `name` and returns its result.
///
/// The exported span is named `name` (via the `otel.name` field) and also carries
/// a `name` attribute for compatibility with 0.1.x dashboards.
///
/// ```
/// use rust_mobile_telemetry::with_span;
///
/// let total = with_span("sum_cart", || 19 + 23);
/// assert_eq!(total, 42);
/// ```
pub fn with_span<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::span!(Level::INFO, "operation", otel.name = name, name = name);
    let _enter = span.enter();
    f()
}

/// Async version of [`with_span`]: awaits the future returned by `f` inside an
/// `INFO` span called `name`.
///
/// ```
/// # #[tokio::main] async fn main() {
/// use rust_mobile_telemetry::with_span_async;
///
/// let user = with_span_async("load_user", || async { "ada" }).await;
/// assert_eq!(user, "ada");
/// # }
/// ```
pub async fn with_span_async<F, Fut, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    use tracing::Instrument;
    let span = tracing::span!(
        Level::INFO,
        "async_operation",
        otel.name = name,
        name = name
    );
    f().instrument(span).await
}

/// Returns a monotonic `u64` counter from the global meter provider.
///
/// Calling it repeatedly with the same name returns the same underlying
/// instrument, so it is fine to call on every request.
pub fn get_counter(name: &str, description: &str) -> Counter<u64> {
    let meter = global::meter(INSTRUMENTATION_SCOPE);
    meter
        .u64_counter(name.to_string())
        .with_description(description.to_string())
        .init()
}

/// Returns an `f64` histogram from the global meter provider (e.g. for latencies
/// in seconds).
pub fn get_histogram(name: &str, description: &str) -> Histogram<f64> {
    let meter = global::meter(INSTRUMENTATION_SCOPE);
    meter
        .f64_histogram(name.to_string())
        .with_description(description.to_string())
        .init()
}

/// Runs `f`, records how long it took (in seconds) to the histogram `name`, and
/// returns its result.
///
/// ```
/// use rust_mobile_telemetry::instrumentation::record_latency;
///
/// let rows = record_latency("db_query_seconds", || vec![1, 2, 3]);
/// assert_eq!(rows.len(), 3);
/// ```
pub fn record_latency<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed().as_secs_f64();

    let histogram = get_histogram(name, "Operation latency");
    histogram.record(duration, &[]);

    result
}
