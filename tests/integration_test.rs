use rust_mobile_telemetry::{ConfigBuilder, ExporterType, init_telemetry};
use rust_mobile_telemetry::instrumentation::{with_span, get_counter};

#[tokio::test]
async fn test_initialization_and_instrumentation() {
    // 1. Setup Config (use Console exporter to avoid network requirements in tests)
    let config = ConfigBuilder::new()
        .service_name("test-service")
        .exporter_type(ExporterType::Console)
        .build();

    // 2. Initialize
    let init_result = init_telemetry(config);
    assert!(init_result.is_ok());

    // 3. Test Instrumentation
    let counter = get_counter("test_counter", "A test counter");
    
    with_span("test_span", || {
        counter.add(1, &[]);
    });

    // If we reached here without panic, basic API usage works.
    // Verifying actual export would require a Mock Exporter or capturing stdout, 
    // which is more complex for a basic smoke test.
}
