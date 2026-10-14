# rust_mobile_telemetry — one-call OpenTelemetry tracing and metrics for Rust mobile backends and edge services

[![Crates.io](https://img.shields.io/crates/v/rust_mobile_telemetry.svg)](https://crates.io/crates/rust_mobile_telemetry)
[![docs.rs](https://img.shields.io/docsrs/rust_mobile_telemetry)](https://docs.rs/rust_mobile_telemetry)
[![License: MIT](https://img.shields.io/crates/l/rust_mobile_telemetry.svg)](https://github.com/emorilebo/rust_mobile_telemetry/blob/main/LICENSE)
[![Downloads](https://img.shields.io/crates/d/rust_mobile_telemetry.svg)](https://crates.io/crates/rust_mobile_telemetry)
[![CI](https://github.com/emorilebo/rust_mobile_telemetry/actions/workflows/ci.yml/badge.svg)](https://github.com/emorilebo/rust_mobile_telemetry/actions/workflows/ci.yml)

Your API works perfectly right up until someone on Android 14 says it doesn't,
and all you have is a log line that says `ok`.

`rust_mobile_telemetry` wires up [OpenTelemetry](https://opentelemetry.io/) for a Rust
service in one function call: `tracing` spans and metrics, exported as JSON to stdout
or over OTLP/gRPC to a collector, tagged with your service name and optional app/device
metadata, and flushed when you shut down.

## What it does

- **One call to set up, one call to flush.** `init_telemetry(config)` installs the
  `tracing` subscriber and the global OpenTelemetry tracer and meter providers;
  `shutdown_telemetry()` exports whatever is still buffered.
- **Two exporters.** `Console` prints JSON lines to stdout for local development;
  `Otlp` sends spans and metrics over gRPC to an OpenTelemetry Collector, Jaeger,
  Grafana Tempo or anything else that speaks OTLP/gRPC.
- **App and device metadata.** `app.version`, `device.os` and `device.id` are added to
  every span and metric, but only the ones you set.
- **Small instrumentation helpers.** `with_span`, `with_span_async`, `get_counter`,
  `get_histogram`, `record_latency`. Your existing `tracing` spans
  (`info_span!`, `#[tracing::instrument]`) are exported too.
- **Works from sync and async code.** Export runs on a small background runtime
  owned by the crate, so it works in a plain `fn main` with no Tokio runtime of your
  own, under `#[tokio::main]`, and in a default (`current_thread`) `#[tokio::test]`.
- **A tested version set.** It pins a compatible set of OpenTelemetry crates
  (see [FAQ](#which-opentelemetry-crate-versions-work-together-with-tracing-opentelemetry)),
  so you don't have to work that out yourself.

## Install

```sh
cargo add rust_mobile_telemetry
# only needed if you want #[tokio::main] like the examples below
cargo add tokio --features macros,rt-multi-thread
```

## Quick start

```rust
use rust_mobile_telemetry::instrumentation::{get_counter, get_histogram};
use rust_mobile_telemetry::{
    init_telemetry, shutdown_telemetry, with_span, ConfigBuilder, ExporterType, MobileMetadata,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ConfigBuilder::new()
        .service_name("checkout-api")
        .exporter_type(ExporterType::Console) // ExporterType::Otlp in production
        .mobile_metadata(MobileMetadata {
            app_version: Some("2.4.1".into()),
            device_os: Some("Android 14".into()),
            device_id: None, // not exported unless you set it
        })
        .build();
    init_telemetry(config)?;

    let orders = get_counter("orders_total", "Orders placed");
    let latency = get_histogram("checkout_seconds", "Checkout duration");

    with_span("place_order", || {
        orders.add(1, &[]);
        latency.record(0.042, &[]);
    });

    shutdown_telemetry(); // flush the last spans and metric values
    Ok(())
}
```

The repository ships the same idea as a runnable example:

```sh
cargo run --example basic_usage
```

Real output (JSON lines truncated here; each one is a full OTLP-shaped payload):

```text
Telemetry initialized. Running operations...
Processing request 1
{"resourceMetrics":{"resource":{"attributes":[...]},"scopeMetrics":[...{"name":"http_requests_total",...
Processing request 2
{"resourceSpans":[{"resource":{"attributes":[{"key":"device.id",...},{"key":"service.name",...}, ...
Processing request 3
...
Operations complete. Shutting down...
{"resourceMetrics":{"resource":{"attributes":[{"key":"device.id","value":{"stringValue":"device-123"}}, ...
```

In that run the three spans were exported as `process_request_1..3`, and the final
metrics export (triggered by `shutdown_telemetry`) carried `http_requests_total = 3`
and a 3-sample `http_request_duration_seconds` histogram. Not pretty, but it works on
a plane with no collector in sight.

## Usage

### Spans

```rust
use rust_mobile_telemetry::{with_span, with_span_async};

#[tokio::main]
async fn main() {
    // Sync: the span is named "parse_receipt" and lasts as long as the closure.
    let total = with_span("parse_receipt", || 19 + 23);

    // Async: the span covers the whole future.
    let profile = with_span_async("load_profile", || async { "ada" }).await;

    // Plain `tracing` works as well; it goes through the same subscriber.
    tracing::info_span!("sync_contacts", batch = 50).in_scope(|| {
        println!("{total} {profile}");
    });
}
```

`with_span` and `with_span_async` create `INFO`-level spans. Which spans are exported
is decided by `RUST_LOG` (an [`EnvFilter`](https://docs.rs/tracing-subscriber/0.3/tracing_subscriber/filter/struct.EnvFilter.html)
directive); when it is unset the filter is `info`.

### Metrics

```rust
use rust_mobile_telemetry::instrumentation::{get_counter, get_histogram, record_latency};
use opentelemetry::KeyValue;

let logins = get_counter("logins_total", "Successful logins");
logins.add(1, &[KeyValue::new("platform", "ios")]);

let payload = get_histogram("payload_bytes", "Request payload size");
payload.record(512.0, &[]);

// Times the closure and records seconds into the histogram "db_query_seconds".
let rows = record_latency("db_query_seconds", || vec![1, 2, 3]);
assert_eq!(rows.len(), 3);
```

Attributes use `opentelemetry::KeyValue` from **opentelemetry 0.21**
(`cargo add opentelemetry@0.21`). Calling `get_counter` / `get_histogram` repeatedly
with the same name is fine; the SDK returns the same instrument.

### Shutdown

Call `shutdown_telemetry()` once, right before the process exits. It blocks until the
final export is done. Skip it and the last few seconds of telemetry leave with the
process, which is usually the part you wanted.

## Configuration

| Builder method | Default | Environment | Notes |
|---|---|---|---|
| `.service_name(&str)` | `"unknown_service"` | `OTEL_SERVICE_NAME` | Env value is applied only if you call `.with_env()`. |
| `.exporter_type(ExporterType)` | `Console` | – | `Console` (stdout JSON) or `Otlp` (gRPC). |
| `.otlp_endpoint(&str)` | `http://localhost:4317` | `OTEL_EXPORTER_OTLP_ENDPOINT`, `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`, `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` | The env vars are read by `opentelemetry-otlp` itself and **override** the builder value whenever they are set. |
| `.enable_tracing(bool)` | `true` | `RUST_LOG` | `RUST_LOG` filters spans; unset or invalid means `info`. |
| `.enable_metrics(bool)` | `true` | `OTEL_METRIC_EXPORT_INTERVAL` | Console metrics are pushed every 60 s (env value in milliseconds overrides), OTLP every 3 s, and both on shutdown. |
| `.mobile_metadata(MobileMetadata)` | all `None` | – | Exported as `app.version`, `device.os`, `device.id` resource attributes. |
| `.with_env()` | – | see above | Copies `OTEL_SERVICE_NAME` and `OTEL_EXPORTER_OTLP_ENDPOINT` into the config. |

`OTEL_EXPORTER_OTLP_TIMEOUT` (seconds, default 10) is also honoured by the OTLP exporter.

Production-style configuration:

```rust,no_run
use rust_mobile_telemetry::{init_telemetry, ConfigBuilder, ExporterType};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = ConfigBuilder::new()
        .service_name("production-backend")
        .exporter_type(ExporterType::Otlp)
        .otlp_endpoint("http://otel-collector:4317")
        .with_env() // let OTEL_SERVICE_NAME / OTEL_EXPORTER_OTLP_ENDPOINT override
        .build();
    init_telemetry(config)?;
    // ... run the service, then:
    rust_mobile_telemetry::shutdown_telemetry();
    Ok(())
}
```

### Errors

`init_telemetry` returns `Err` (it does not panic) when it has already been called in
this process, when another global `tracing` subscriber or `log` logger is installed,
when the OTLP endpoint is not a valid URI, or when the background export runtime
cannot start. An unreachable collector is not an init error: export failures are
reported later through OpenTelemetry's global error handler (stderr by default).

## How it works

1. A `Resource` is built from `service.name` plus whichever `MobileMetadata` fields are set.
2. With tracing enabled, a `tracing_subscriber::Registry` with an `EnvFilter` and a
   `tracing-opentelemetry` layer becomes the global subscriber. `Console` uses a simple
   (export-on-end) span processor; `Otlp` uses a batch processor.
3. With metrics enabled, an OpenTelemetry `MeterProvider` with a periodic reader becomes
   the global meter provider, and a handle is kept for shutdown.
4. Batching, periodic collection and the gRPC client run on a one-worker Tokio runtime
   owned by the crate (thread name `rust-mobile-telemetry`). That is why it works from
   sync code and why `shutdown_telemetry()` can block without deadlocking your runtime.

No magic: a few global singletons and a thread named after the crate.

## Privacy: what gets exported

- Nothing about the device or user is collected automatically. The resource carries
  `service.name` and only the `MobileMetadata` fields you set; with the default config
  that means `service.name` alone.
- Spans also carry attributes added by `tracing-opentelemetry`: `code.filepath`,
  `code.namespace`, `code.lineno`, `thread.id`, `thread.name`, `busy_ns`, `idle_ns`,
  plus every field you record on your spans. Don't put tokens, emails or other
  personal data in span fields.
- `device_id` is a persistent identifier and may count as personal data under laws
  like the GDPR. If you need one, send a pseudonymous value.
- `Otlp` sends data in plain text (TLS is not enabled in this version), so point it at
  a collector on localhost or inside your private network.

## Limitations

- **Pinned to an older OpenTelemetry stack**: opentelemetry / opentelemetry_sdk 0.21,
  opentelemetry-otlp 0.14, opentelemetry-stdout 0.2, tracing-opentelemetry 0.22.
  Instruments returned by `get_counter` / `get_histogram` are 0.21 types and won't
  mix with a different OpenTelemetry version in your own code.
- **Known advisories in that stack** (upstream fixes need the upgrade on the roadmap):
  - `opentelemetry_sdk` < 0.32.1, GHSA-w9wp-h8wv-79jx: unbounded memory allocation in
    W3C Baggage propagation. This crate never installs a propagator, so it only
    applies if your application installs the `BaggagePropagator` itself.
  - `h2` 0.3 (via `tonic` 0.9), RUSTSEC-2026-0258: unbounded empty DATA frames. It
    affects the gRPC connection to the collector; another reason to use a collector
    you control.
- **OTLP over gRPC only**, without TLS or custom headers. Vendors that need an API-key
  header (most hosted back ends) are reached through an OpenTelemetry Collector that
  forwards the data.
- **Sampling is always-on**: every span that passes `RUST_LOG` is exported.
- **Metadata is per process**, not per request. For a backend serving many devices,
  record device details as span fields instead of `MobileMetadata`.
- **`init_telemetry` works once per process**, because the global `tracing`
  subscriber can only be set once.
- `Console` output shares stdout with your own `println!`s.
- The `instrumentation` cargo feature is a deprecated no-op kept for compatibility.

## Roadmap

- Upgrade to OpenTelemetry 0.32.1+ (current upstream is 0.33). This is a breaking change
  because the public API exposes OpenTelemetry types, and OpenTelemetry Rust has
  shipped twelve minor releases since 0.21. Reading all twelve changelogs counts as a hobby.
- TLS and custom headers for the OTLP exporter, and OTLP over HTTP.
- Configurable sampler.
- Remove the deprecated `instrumentation` feature.

## FAQ

### How do I add OpenTelemetry tracing and metrics to a Rust backend with one function call?

Add `rust_mobile_telemetry`, build a `Config` with `ConfigBuilder`, and call
`init_telemetry(config)` at startup. It installs the `tracing` subscriber and the global
OpenTelemetry tracer and meter providers in one step. Call `shutdown_telemetry()`
before exit to flush.

### How do I send Rust traces and metrics to an OpenTelemetry Collector over OTLP?

Use `.exporter_type(ExporterType::Otlp)` and `.otlp_endpoint("http://collector:4317")`, or
set `OTEL_EXPORTER_OTLP_ENDPOINT`. rust_mobile_telemetry then exports spans in batches and
metrics every 3 seconds over plain-text gRPC. The test suite checks this end to end
against an in-process OTLP gRPC server.

### Why are my Rust OpenTelemetry metrics missing when the program exits?

Periodic metric readers only export on a timer, so a short-lived process exits before
the next tick. rust_mobile_telemetry's `shutdown_telemetry()` force-flushes the meter
provider and shuts down the tracer provider, so the final values are exported.
(It also works around opentelemetry_sdk 0.21, where `MeterProvider::shutdown()` alone
exports nothing.)

### Which OpenTelemetry crate versions work together with tracing-opentelemetry?

This crate uses opentelemetry 0.21, opentelemetry_sdk 0.21, opentelemetry-otlp 0.14,
opentelemetry-stdout 0.2 and tracing-opentelemetry 0.22. They resolve to a single
`opentelemetry` 0.21 in the dependency tree, and the test suite exports through them
end to end. Mixing releases from different lines compiles into separate, incompatible
types, which is the usual cause of "trait bound not satisfied" errors.

### Does rust_mobile_telemetry collect device IDs or other personal data?

No, not by itself. It exports `service.name` plus only the `MobileMetadata` fields you set,
and the attributes on your spans. A device ID is exported only if you put one in
`MobileMetadata::device_id`.

### Why does `RUST_LOG` change which spans are exported?

The global subscriber filters spans with `tracing_subscriber::EnvFilter`, which reads
`RUST_LOG`. When it is unset, rust_mobile_telemetry uses `info`, which keeps the `INFO`
spans created by `with_span`. `RUST_LOG=warn` would drop them.

## Contributing

Issues and pull requests are welcome at
[github.com/emorilebo/rust_mobile_telemetry](https://github.com/emorilebo/rust_mobile_telemetry).
Before opening a PR:

```sh
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

The OTLP test spins up its own gRPC collector on `127.0.0.1`, so no network or Docker
is needed. README code blocks are compiled as doctests.

## Related projects by the same author

- [g-recon](https://crates.io/crates/g-recon): security scanner for GitHub repos and web reconnaissance.
- [npm_sentinel](https://crates.io/crates/npm_sentinel): CLI that detects supply-chain attacks in npm packages by analysing lifecycle scripts and dependencies.
- [rust_mobile_secrets_vault](https://crates.io/crates/rust_mobile_secrets_vault): encrypted secrets vault with versioning and key rotation for mobile-backend and embedded Rust services.
- [rust_secure_dependency_audit](https://crates.io/crates/rust_secure_dependency_audit): audits Rust dependencies for health, licences and maintenance risk.

## About the author

> Built by **[Godfrey Lebo](https://godfreylebo.dev)** — Senior Backend & Product Engineer and founder of Sether.
> I build the backend, connect the product, and stay responsible for what happens after launch —
> lately that means open-source tooling for AI security and software supply-chain security.
>
> [Website](https://godfreylebo.dev) · [GitHub](https://github.com/emorilebo) · [X / Twitter](https://x.com/emori_lebo) · [LinkedIn](https://www.linkedin.com/in/godfreylebo) · [Email](mailto:emorylebo@gmail.com)

## License

MIT. See [LICENSE](LICENSE).
