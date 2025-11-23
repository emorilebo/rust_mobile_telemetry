# Rust Mobile Telemetry

`rust_mobile_telemetry` is a production-quality Rust crate providing observability instrumentation (tracing + metrics) tailored for mobile-backend applications or embedded/edge services. It leverages the OpenTelemetry ecosystem.

## Features

- **OpenTelemetry Integration**: Built-in Tracer and Meter providers.
- **Easy Configuration**: Builder pattern for service name, OTLP endpoint, and mobile metadata.
- **Instrumentation Helpers**: Simple `with_span` wrapper and metrics helpers.
- **Exporters**: Support for OTLP (gRPC) and Console (Stdout) exporters.
- **Mobile Metadata**: Attach device info (OS, version, ID) to all telemetry.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust_mobile_telemetry = "0.1.0"
```

## Usage

### Initialization

```rust
use rust_mobile_telemetry::{ConfigBuilder, ExporterType, init_telemetry, MobileMetadata};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let metadata = MobileMetadata {
        app_version: Some("1.0.0".to_string()),
        device_os: Some("iOS 17".to_string()),
        device_id: Some("device-uuid".to_string()),
    };

    let config = ConfigBuilder::new()
        .service_name("my-mobile-service")
        .otlp_endpoint("http://localhost:4317")
        .exporter_type(ExporterType::Otlp)
        .mobile_metadata(metadata)
        .build();

    init_telemetry(config)?;
    
    // ... application logic ...
    
    Ok(())
}
```

### Instrumentation

```rust
use rust_mobile_telemetry::with_span;
use rust_mobile_telemetry::instrumentation::get_counter;

fn perform_task() {
    let counter = get_counter("task_count", "Number of tasks performed");
    
    with_span("perform_task", || {
        // Your logic here
        counter.add(1, &[]);
    });
}
```

## Configuration

You can also configure via environment variables if using `ConfigBuilder::with_env()`:

- `OTEL_SERVICE_NAME`
- `OTEL_EXPORTER_OTLP_ENDPOINT`

## License

MIT
