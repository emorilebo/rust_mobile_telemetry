//! Prints spans and metrics to stdout as JSON lines.
//!
//! Run with: `cargo run --example basic_usage`

use rust_mobile_telemetry::instrumentation::{get_counter, get_histogram};
use rust_mobile_telemetry::{
    init_telemetry, shutdown_telemetry, with_span, ConfigBuilder, ExporterType, MobileMetadata,
};
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Configure Telemetry
    let metadata = MobileMetadata {
        app_version: Some("1.0.0".to_string()),
        device_os: Some("Android 14".to_string()),
        device_id: Some("device-123".to_string()),
    };

    let config = ConfigBuilder::new()
        .service_name("mobile-backend-example")
        .exporter_type(ExporterType::Console) // Use Console for this example
        .mobile_metadata(metadata)
        .build();

    // 2. Initialize
    init_telemetry(config)?;

    println!("Telemetry initialized. Running operations...");

    // 3. Instrument Operations
    let request_counter = get_counter("http_requests_total", "Total number of HTTP requests");
    let latency_histogram = get_histogram("http_request_duration_seconds", "HTTP request duration");

    for i in 1..=3 {
        with_span(&format!("process_request_{}", i), || {
            println!("Processing request {}", i);

            request_counter.add(1, &[]);

            let start = std::time::Instant::now();

            // Simulate work
            thread::sleep(Duration::from_millis(100 * i));

            let duration = start.elapsed().as_secs_f64();
            latency_histogram.record(duration, &[]);
        });
    }

    println!("Operations complete. Shutting down...");

    // 4. Flush the last spans and the final metric values before exiting.
    shutdown_telemetry();

    Ok(())
}
