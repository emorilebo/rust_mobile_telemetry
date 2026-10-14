//! Regression test: 0.1.3 panicked if the application had already installed a
//! global `tracing` subscriber. `init_telemetry` must return an `Err` instead.

use rust_mobile_telemetry::{init_telemetry, ConfigBuilder};

#[test]
fn returns_error_when_a_global_subscriber_already_exists() {
    tracing::subscriber::set_global_default(tracing_subscriber::registry())
        .expect("test installs the first subscriber");

    let result = init_telemetry(ConfigBuilder::new().enable_metrics(false).build());

    let message = result.expect_err("init must fail, not panic").to_string();
    assert!(message.contains("tracing subscriber"), "{message}");
}
