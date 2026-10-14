//! Regression tests for 0.1.3 panics: `init_telemetry` panicked when called
//! outside a Tokio runtime ("there is no reactor running") and when called a
//! second time ("failed to set global default subscriber"). Both must now work
//! or return an `Err`.

use rust_mobile_telemetry::instrumentation::get_counter;
use rust_mobile_telemetry::{init_telemetry, shutdown_telemetry, with_span, ConfigBuilder};

#[test]
fn works_without_a_tokio_runtime_and_rejects_a_second_init() {
    // Plain synchronous test: no Tokio runtime anywhere.
    let first = init_telemetry(ConfigBuilder::new().service_name("sync-app").build());
    assert!(first.is_ok(), "first init failed: {first:?}");

    let second = init_telemetry(ConfigBuilder::new().build());
    let message = second.expect_err("second init must fail").to_string();
    assert!(message.contains("already been called"), "{message}");

    with_span("sync_work", || {
        get_counter("sync_jobs_total", "Jobs").add(1, &[])
    });
    shutdown_telemetry();
}
