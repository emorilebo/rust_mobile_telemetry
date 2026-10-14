//! Regression test: `shutdown_telemetry` blocks until the final export is done.
//! When export tasks ran on the caller's runtime, a `current_thread` runtime
//! (e.g. a plain `#[tokio::test]`) deadlocked here. Export now runs on a
//! crate-owned runtime, so this must return promptly.

use std::time::Duration;

use rust_mobile_telemetry::instrumentation::get_histogram;
use rust_mobile_telemetry::{init_telemetry, shutdown_telemetry, with_span_async, ConfigBuilder};

#[tokio::test]
async fn shutdown_does_not_deadlock_on_a_current_thread_runtime() {
    // Fail loudly instead of hanging CI forever if the deadlock comes back.
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(30));
        eprintln!("shutdown_telemetry deadlocked on a current_thread runtime");
        std::process::abort();
    });

    init_telemetry(ConfigBuilder::new().service_name("current-thread").build())
        .expect("init_telemetry should succeed");

    with_span_async("load_feed", || async {
        get_histogram("feed_load_seconds", "Feed load time").record(0.1, &[]);
    })
    .await;

    shutdown_telemetry();
}
