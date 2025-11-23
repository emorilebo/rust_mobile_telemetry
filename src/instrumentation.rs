use tracing::Level;
use opentelemetry::global;
use opentelemetry::metrics::Counter;
use opentelemetry::metrics::Histogram;
use std::time::Instant;
use std::future::Future;

/// Wraps a synchronous operation in a span.
pub fn with_span<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let span = tracing::span!(Level::INFO, "operation", name = name);
    let _enter = span.enter();
    f()
}

/// Wraps an async operation in a span.
pub async fn with_span_async<F, Fut, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    let span = tracing::span!(Level::INFO, "async_operation", name = name);
    use tracing::Instrument;
    f().instrument(span).await
}

/// Helper to get a counter metric.
pub fn get_counter(name: &str, description: &str) -> Counter<u64> {
    let meter = global::meter("rust_mobile_telemetry");
    meter
        .u64_counter(name.to_string())
        .with_description(description.to_string())
        .init()
}

/// Helper to get a histogram metric (e.g. for latency).
pub fn get_histogram(name: &str, description: &str) -> Histogram<f64> {
    let meter = global::meter("rust_mobile_telemetry");
    meter
        .f64_histogram(name.to_string())
        .with_description(description.to_string())
        .init()
}

/// Records the duration of a closure execution to a histogram.
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
