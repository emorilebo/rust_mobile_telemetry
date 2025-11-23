# Rust Mobile Telemetry 🚀

[![Crates.io](https://img.shields.io/crates/v/rust_mobile_telemetry.svg)](https://crates.io/crates/rust_mobile_telemetry)
[![Documentation](https://docs.rs/rust_mobile_telemetry/badge.svg)](https://docs.rs/rust_mobile_telemetry)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Observability made simple for mobile backends and edge services.**

`rust_mobile_telemetry` is a production-ready Rust crate that helps you understand *exactly* what is happening in your application. It provides **tracing** (timelines of operations) and **metrics** (counters, gauges) with minimal setup, tailored for mobile-backend environments where device metadata (like OS version or Device ID) is critical for debugging.

## Why use this?

- **🔍 Instant Visibility**: See full request traces and performance metrics immediately.
- **📱 Mobile-First**: Automatically attaches mobile metadata (App Version, OS, Device ID) to every trace.
- **⚡ Zero-Config Defaults**: Works out of the box with sensible defaults, but fully customizable.
- **🔌 OpenTelemetry Standard**: Built on the industry-standard OpenTelemetry ecosystem, compatible with Jaeger, Prometheus, Honeycomb, Datadog, and more.

---

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust_mobile_telemetry = "0.1.1"
```

---

## 🚀 Getting Started

### 1. Initialize Telemetry

Call `init_telemetry` at the start of your `main` function. This sets up the global tracer and meter.

```rust
use rust_mobile_telemetry::{ConfigBuilder, ExporterType, init_telemetry, MobileMetadata};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Define metadata about the client (optional but recommended)
    let metadata = MobileMetadata {
        app_version: Some("1.0.0".to_string()),
        device_os: Some("iOS 17.2".to_string()),
        device_id: Some("user-device-uuid-123".to_string()),
    };

    // 2. Build configuration
    let config = ConfigBuilder::new()
        .service_name("my-mobile-backend")
        .exporter_type(ExporterType::Console) // Use Console for local dev, Otlp for production
        .mobile_metadata(metadata)
        .build();

    // 3. Initialize!
    init_telemetry(config)?;
    
    println!("Telemetry initialized! 📡");
    
    // ... your app logic ...
    
    // 4. Ensure data is flushed before exit
    rust_mobile_telemetry::init::shutdown_telemetry();
    
    Ok(())
}
```

### 2. Instrument Your Code

Wrap operations in spans to track how long they take.

```rust
use rust_mobile_telemetry::with_span;

fn process_user_login() {
    with_span("process_login", || {
        // Your logic here...
        println!("Logging in user...");
        std::thread::sleep(std::time::Duration::from_millis(50));
    });
}
```

### 3. Record Metrics

Count events or measure latencies.

```rust
use rust_mobile_telemetry::instrumentation::{get_counter, get_histogram};

fn handle_request() {
    // Counters: Good for "how many times did X happen?"
    let requests = get_counter("http_requests_total", "Total HTTP requests");
    requests.add(1, &[]);

    // Histograms: Good for "how long did X take?"
    let latency = get_histogram("http_request_duration", "Request duration in seconds");
    latency.record(0.123, &[]);
}
```

---

## ⚙️ Configuration Options

You can configure the library using the `ConfigBuilder` or Environment Variables.

| Option | Builder Method | Env Variable | Description |
|--------|---------------|--------------|-------------|
| **Service Name** | `.service_name("name")` | `OTEL_SERVICE_NAME` | Name of your service (e.g., `auth-api`). |
| **Exporter** | `.exporter_type(type)` | N/A | Where to send data: `Console` (stdout) or `Otlp` (collector). |
| **OTLP Endpoint** | `.otlp_endpoint("url")` | `OTEL_EXPORTER_OTLP_ENDPOINT` | URL for OTLP exporter (default: `http://localhost:4317`). |
| **Metrics** | `.enable_metrics(bool)` | N/A | Enable/disable metrics collection. |
| **Tracing** | `.enable_tracing(bool)` | N/A | Enable/disable tracing. |

### Example: Production Config (OTLP)

For production, you typically want to send data to an OpenTelemetry Collector or a vendor (like Honeycomb/Datadog).

```rust
let config = ConfigBuilder::new()
    .service_name("production-backend")
    .exporter_type(ExporterType::Otlp) // Send over network
    .otlp_endpoint("http://otel-collector:4317")
    .build();
```

---

## ❓ Troubleshooting

### "File locking" error on Windows
If you see `os error 32` during build or publish, it's likely an antivirus or the OS locking the build directory.
**Fix**: Restart your terminal or computer.

### No output in Console?
Ensure you initialized with `ExporterType::Console` and that your application runs long enough to flush data (or call `shutdown_telemetry()`).

---

## 📄 License

MIT License. See [LICENSE](LICENSE) for details.
